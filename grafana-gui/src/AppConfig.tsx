import React, { Component, ChangeEvent, ReactNode, MouseEvent } from "react";
import {
    AsyncSelect,
    SecretInput,
    Stack,
    Button,
    Field,
    Modal,
    Input,
    FieldSet,
    Alert,
} from "@grafana/ui";
import {
    AppPluginMeta,
    PluginConfigPageProps,
    PluginMeta,
    SelectableValue,
    createTheme,
} from "@grafana/data";
import { getBackendSrv, locationService } from "@grafana/runtime";
import { Ajv, JSONSchemaType } from "ajv";
import { css } from "@emotion/css";
import { lastValueFrom } from "rxjs";

import { init_i18n, t } from "./i18n";
import { EmpowerdApi, GraphQlError } from "./control/EmpowerdApi";
init_i18n();

export type WeatherLabels = {
    x1: string | null;
    x2: string | null;
    x3: string | null;
    x4: string | null;
    x5: string | null;
    x6: string | null;
    x7: string | null;
};

export const WeatherLabelsDefault = {
    x1: null,
    x2: null,
    x3: null,
    x4: null,
    x5: null,
    x6: null,
    x7: null,
};

export type Ranges = {
    production: Array<number | null>;
    consumption: number | null;
    battery: Array<number | null>;
    boiler: Array<number | null>;
    heating: number | null;
    cop: number | null;
};

export const RangesDefault = {
    production: [null, null],
    consumption: null,
    battery: [null, null],
    boiler: [null, null],
    heating: null,
    cop: null,
};

export type BackendConfig = {
    batteries: number[];
    controls: boolean;
    generators: number[];
    heatpumps: number[];
    meters: number[];
    solars: number[];
    wallboxes: number[];
    weathers: number[];
    labels: WeatherLabels;
    ranges: Ranges;
};

export const BackendConfigDefault = {
    batteries: [],
    controls: true,
    generators: [],
    heatpumps: [],
    meters: [],
    solars: [],
    wallboxes: [],
    weathers: [],
    labels: WeatherLabelsDefault,
    ranges: RangesDefault,
};

export type ConfigJson = {
    apiLocation?: string;
    datasource?: PsqlDatasource;
    backend?: BackendConfig;
};

type AppConfigStyles = {
    colorWeak: string;
    marginTop: string;
    marginTopXl: string;
};

type PsqlDatasource = {
    name: string;
    uid: string;
};

type AnyDatasource = {
    name: string;
    typeName: string;
    uid: string;
};

interface AppConfigProps
    extends PluginConfigPageProps<AppPluginMeta<ConfigJson>> {}

type AppConfigState = {
    apiLocation: string;
    datasource: SelectableValue<PsqlDatasource>;
    backend: BackendConfig;
    backend_str: string;
    showModal: boolean;
    apiBusy: boolean;
    apiError: string;
    apiUsername: string;
    apiPassword: string;
};

const BackendSchema: JSONSchemaType<BackendConfig> = {
    type: "object",
    definitions: {
        idArray: {
            type: "array",
            items: {
                type: "integer",
            },
        },
        optNumber: {
            type: "number",
            nullable: true,
        },
        optString: {
            type: "string",
            nullable: true,
        },
        minMaxArray: {
            type: "array",
            items: { type: "number", nullable: true },
            minItems: 2,
            maxItems: 2,
        },
        ranges: {
            type: "object",
            properties: {
                production: { $ref: "#/definitions/minMaxArray" },
                consumption: { $ref: "#/definitions/optNumber" },
                battery: { $ref: "#/definitions/minMaxArray" },
                boiler: { $ref: "#/definitions/minMaxArray" },
                heating: { $ref: "#/definitions/optNumber" },
                cop: { $ref: "#/definitions/optNumber" },
            },
            required: ["production", "battery", "boiler"],
            additionalProperties: false,
        },
        weatherLabels: {
            type: "object",
            properties: {
                x1: { $ref: "#/definitions/optString" },
                x2: { $ref: "#/definitions/optString" },
                x3: { $ref: "#/definitions/optString" },
                x4: { $ref: "#/definitions/optString" },
                x5: { $ref: "#/definitions/optString" },
                x6: { $ref: "#/definitions/optString" },
                x7: { $ref: "#/definitions/optString" },
            },
            required: ["x1", "x2", "x3", "x4", "x5", "x6", "x7"],
            additionalProperties: false,
        },
    },
    properties: {
        batteries: { $ref: "#/definitions/idArray" },
        controls: { type: "boolean" },
        generators: { $ref: "#/definitions/idArray" },
        heatpumps: { $ref: "#/definitions/idArray" },
        meters: { $ref: "#/definitions/idArray" },
        solars: { $ref: "#/definitions/idArray" },
        wallboxes: { $ref: "#/definitions/idArray" },
        weathers: { $ref: "#/definitions/idArray" },
        labels: { $ref: "#/definitions/weatherLabels" },
        ranges: { $ref: "#/definitions/ranges" },
    },
    required: [
        "batteries",
        "controls",
        "generators",
        "heatpumps",
        "meters",
        "solars",
        "wallboxes",
        "weathers",
        "labels",
        "ranges",
    ],
    additionalProperties: false,
};

export class AppConfig extends Component<AppConfigProps, AppConfigState> {
    private styles: AppConfigStyles;
    private backend_cfg_validator;
    private api: EmpowerdApi;

    constructor(props: AppConfigProps) {
        super(props);
        const { jsonData } = props.plugin.meta;

        this.styles = this.getStyles();
        this.backend_cfg_validator = new Ajv({ allErrors: true }).compile(
            BackendSchema
        );
        this.api = new EmpowerdApi(jsonData?.apiLocation || "");
        this.state = {
            apiLocation: jsonData?.apiLocation || "",
            datasource: {
                label: jsonData?.datasource?.name || "",
                value: jsonData?.datasource || { name: "", uid: "" },
            },
            backend: jsonData?.backend || BackendConfigDefault,
            backend_str: JSON.stringify(
                jsonData?.backend || BackendConfigDefault
            ),
            showModal: false,
            apiBusy: false,
            apiError: "",
            apiUsername: "",
            apiPassword: "",
        };
    }

    private getStyles(): AppConfigStyles {
        let theme = createTheme({ colors: { mode: "dark" } });

        return {
            colorWeak: css`
                color: ${theme.colors.text.secondary};
            `,
            marginTop: css`
                margin-top: ${theme.spacing(3)};
            `,
            marginTopXl: css`
                margin-top: ${theme.spacing(6)};
            `,
        };
    }

    public onChangeApiLocation(event: ChangeEvent<HTMLInputElement>) {
        this.api = new EmpowerdApi(event.target.value.trim());
        this.setState({
            ...this.state,
            apiLocation: event.target.value.trim(),
        });
    }

    public onChangeDatasource(event: SelectableValue<PsqlDatasource>) {
        this.setState({
            ...this.state,
            datasource: event,
        });
    }

    public onChangeConfig(event: ChangeEvent<HTMLInputElement>) {
        this.setState({
            ...this.state,
            backend_str: event.target.value.trim(),
        });
    }

    public onEnable(_event: MouseEvent<HTMLButtonElement>) {
        this.updatePluginAndReload(this.props.plugin.meta.id, {
            enabled: true,
            pinned: true,
            jsonData: this.props.plugin.meta.jsonData,
        });
    }

    public onDisable(_event: MouseEvent<HTMLButtonElement>) {
        this.updatePluginAndReload(this.props.plugin.meta.id, {
            enabled: false,
            pinned: false,
            jsonData: this.props.plugin.meta.jsonData,
        });
    }

    public onSubmit(_event: MouseEvent<HTMLButtonElement>) {
        const { enabled, id, pinned } = this.props.plugin.meta;

        this.updatePluginAndReload(id, {
            enabled,
            pinned,
            jsonData: {
                apiLocation: this.state.apiLocation,
                datasource: this.state.datasource.value,
                // Validated before submit is allowed
                backend: JSON.parse(this.state.backend_str),
            },
        });
    }

    public onShowLoadSettings(_event: MouseEvent<HTMLButtonElement>) {
        this.setState({
            ...this.state,
            showModal: true,
        });
    }

    public onCloseModal() {
        this.setState({
            ...this.state,
            showModal: false,
        });
    }

    public onChangeApiUsername(event: ChangeEvent<HTMLInputElement>) {
        this.setState({
            ...this.state,
            apiUsername: event.target.value,
        });
    }

    public onChangeApiPassword(event: ChangeEvent<HTMLInputElement>) {
        this.setState({
            ...this.state,
            apiPassword: event.target.value,
        });
    }

    public onResetApiPassword() {
        this.setState({
            ...this.state,
            apiPassword: "",
        });
    }

    public onLoadSettings(_event: MouseEvent<HTMLButtonElement>) {
        if (this.state.apiBusy) {
            return;
        }
        this.setState({
            ...this.state,
            apiBusy: true,
        });

        this.api.login(
            this.state.apiUsername,
            this.state.apiPassword,
            () => {
                this.api.backendConfig(
                    (config) => {
                        this.setState({
                            ...this.state,
                            apiBusy: false,
                            apiError: "",
                            backend_str: config,
                            showModal: false,
                        });
                        this.api.logout(
                            () => {},
                            () => {}
                        );
                    },
                    (errors: GraphQlError[]) => {
                        let error = errors
                            .map((x): string => {
                                return x.message;
                            })
                            .toString();
                        this.setState({
                            ...this.state,
                            apiBusy: false,
                            apiError: error,
                        });
                        this.api.logout(
                            () => {},
                            () => {}
                        );
                    }
                );
            },
            (errors: GraphQlError[]) => {
                let error = errors
                    .map((x): string => {
                        return x.message;
                    })
                    .toString();
                this.setState({
                    ...this.state,
                    apiBusy: false,
                    apiError: error,
                });
            }
        );
    }

    private backendCfgValid(): Boolean {
        try {
            const cfg = JSON.parse(this.state.backend_str);
            if (!this.backend_cfg_validator(cfg)) {
                console.log(
                    `Validation failed: ` +
                        JSON.stringify(this.backend_cfg_validator.errors)
                );
                return false;
            }
            return true;
        } catch {
            return false;
        }
    }

    private async updatePluginAndReload(
        pluginId: string,
        data: Partial<PluginMeta<ConfigJson>>
    ) {
        try {
            await this.updatePlugin(pluginId, data);

            // Reloading the page as the changes made here wouldn't be
            // propagated to the actual plugin otherwise.
            // This is not ideal, however unfortunately currently there is no
            // supported way for updating the plugin state.
            locationService.reload();
        } catch (e) {
            console.error("Error while updating the plugin", e);
        }
    }

    private async updatePlugin(pluginId: string, data: Partial<PluginMeta>) {
        const request = getBackendSrv().fetch({
            url: `/api/plugins/${pluginId}/settings`,
            method: "POST",
            data,
        });
        const response = await lastValueFrom(request);

        return response.data;
    }

    private async fetchDatasources(): Promise<
        Array<SelectableValue<PsqlDatasource>>
    > {
        const request = getBackendSrv().fetch({
            url: "/api/datasources",
            method: "GET",
        });
        const response = await lastValueFrom(request);

        return (response.data as AnyDatasource[])
            .filter((item: AnyDatasource) => {
                return item.typeName === "PostgreSQL";
            })
            .map((item: AnyDatasource) => {
                return {
                    value: { name: item.name, uid: item.uid },
                    label: item.name,
                };
            });
    }

    private renderEnableDisable(): ReactNode {
        let inner = null;

        if (this.props.plugin.meta.enabled) {
            inner = (
                <>
                    <div className={this.styles.colorWeak}>
                        {t("currently-enabled")}
                    </div>
                    <Button
                        className={this.styles.marginTop}
                        variant="destructive"
                        onClick={this.onEnable.bind(this)}
                    >
                        {t("disable-plugin")}
                    </Button>
                </>
            );
        } else {
            inner = (
                <>
                    <div className={this.styles.colorWeak}>
                        {t("currently-disabled")}
                    </div>
                    <Button
                        className={this.styles.marginTop}
                        variant="primary"
                        onClick={this.onDisable.bind(this)}
                    >
                        {t("enable-plugin")}
                    </Button>
                </>
            );
        }

        return <FieldSet label={t("enable-disable")}>{inner}</FieldSet>;
    }

    private renderApiError(): ReactNode {
        if (this.state.apiError) {
            return (
                <Alert
                    title={t("auto-detect-settings-failed")}
                    severity="error"
                >
                    {this.state.apiError}
                </Alert>
            );
        } else {
            return <></>;
        }
    }

    public render(): ReactNode {
        const cfg_valid = this.backendCfgValid();

        return (
            <div>
                {this.renderEnableDisable()}

                <FieldSet
                    label={t("api-settings")}
                    className={this.styles.marginTopXl}
                >
                    <Field
                        label={t("psql-source")}
                        description={t("psql-source-desc")}
                    >
                        <AsyncSelect
                            loadOptions={this.fetchDatasources}
                            defaultOptions
                            value={this.state.datasource}
                            onChange={this.onChangeDatasource.bind(this)}
                        />
                    </Field>
                    <Field
                        label={t("api-location")}
                        description={t("api-location-desc")}
                        className={this.styles.marginTop}
                    >
                        <Input
                            width={60}
                            label={t("api-location")}
                            value={this.state?.apiLocation}
                            placeholder={`${t("eg")}: /empowerd`}
                            onChange={this.onChangeApiLocation.bind(this)}
                        />
                    </Field>
                    <Field
                        label={t("config-json")}
                        description={t("config-json-desc")}
                        className={this.styles.marginTop}
                    >
                        <Stack>
                            <Input
                                width={60}
                                label={t("config-json")}
                                value={this.state?.backend_str}
                                required
                                invalid={!cfg_valid}
                                onChange={this.onChangeConfig.bind(this)}
                            />
                            <Button
                                variant="secondary"
                                onClick={this.onShowLoadSettings.bind(this)}
                                disabled={!this.state.apiLocation}
                            >
                                {t("auto-detect-settings")}
                            </Button>
                        </Stack>
                    </Field>
                    <div className={this.styles.marginTop}>
                        <Button
                            type="submit"
                            onClick={this.onSubmit.bind(this)}
                            disabled={Boolean(
                                !this.state.apiLocation || !cfg_valid
                            )}
                        >
                            {t("save-settings")}
                        </Button>
                    </div>
                </FieldSet>
                <Modal
                    title={t("auto-detect-settings-modal")}
                    isOpen={this.state.showModal}
                    onDismiss={this.onCloseModal.bind(this)}
                    onClickBackdrop={() => {}}
                >
                    {this.renderApiError()}
                    <div>{t("auto-detect-settings-div")}</div>
                    <Field
                        label={t("username")}
                        description={t("api-username-desc")}
                        className={this.styles.marginTop}
                    >
                        <Input
                            width={60}
                            label={t("username")}
                            value={this.state?.apiUsername}
                            onChange={this.onChangeApiUsername.bind(this)}
                        />
                    </Field>
                    <Field
                        label={t("password")}
                        description={t("api-password-desc")}
                        className={this.styles.marginTop}
                    >
                        <SecretInput
                            width={60}
                            label={t("password")}
                            value={this.state?.apiPassword}
                            isConfigured={false}
                            onChange={this.onChangeApiPassword.bind(this)}
                            onReset={() => {}}
                        />
                    </Field>
                    <Modal.ButtonRow>
                        <Button
                            variant="secondary"
                            onClick={this.onCloseModal.bind(this)}
                        >
                            {t("cancel")}
                        </Button>
                        <Button
                            variant="primary"
                            onClick={this.onLoadSettings.bind(this)}
                            disabled={
                                !this.state.apiUsername ||
                                !this.state.apiPassword ||
                                this.state.apiBusy
                            }
                        >
                            {t("ok")}
                        </Button>
                    </Modal.ButtonRow>
                </Modal>
            </div>
        );
    }
}
