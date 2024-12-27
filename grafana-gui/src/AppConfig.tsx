import React, { Component, ChangeEvent, ReactNode, MouseEvent } from "react";
import { AsyncSelect, Button, Field, Input, FieldSet } from "@grafana/ui";
import {
    AppPluginMeta,
    PluginConfigPageProps,
    PluginMeta,
    SelectableValue,
    createTheme,
} from "@grafana/data";
import { getBackendSrv, locationService } from "@grafana/runtime";
import { css } from "@emotion/css";
import { lastValueFrom } from "rxjs";

export type WeatherLabels = {
    x1: string;
    x2: string;
    x3: string;
};

export const WeatherLabelsDefault = {
    x1: "X1",
    x2: "X2",
    x3: "X3",
};

export type BackendConfig = {
    batteries: number[];
    generators: number[];
    heatpumps: number[];
    meters: number[];
    solars: number[];
    wallboxes: number[];
    weathers: number[];
    labels: WeatherLabels;
};

export const BackendConfigDefault = {
    batteries: [],
    generators: [],
    heatpumps: [],
    meters: [],
    solars: [],
    wallboxes: [],
    weathers: [],
    labels: WeatherLabelsDefault,
};

export type ConfigJson = {
    apiUrl?: string;
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
    apiUrl: string;
    datasource: SelectableValue<PsqlDatasource>;
    backend: BackendConfig;
    backend_str: string;
};

export class AppConfig extends Component<AppConfigProps, AppConfigState> {
    private styles: AppConfigStyles;

    constructor(props: AppConfigProps) {
        super(props);
        const { jsonData } = props.plugin.meta;

        this.styles = this.getStyles();
        this.state = {
            apiUrl: jsonData?.apiUrl || "",
            datasource: {
                label: jsonData?.datasource?.name || "",
                value: jsonData?.datasource || { name: "", uid: "" },
            },
            backend: jsonData?.backend || BackendConfigDefault,
            backend_str: JSON.stringify(
                jsonData?.backend || BackendConfigDefault
            ),
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

    public onChangeApiUrl(event: ChangeEvent<HTMLInputElement>) {
        this.setState({
            ...this.state,
            apiUrl: event.target.value.trim(),
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
                apiUrl: this.state.apiUrl,
                datasource: this.state.datasource.value,
                // Validated before submit is allowed
                backend: JSON.parse(this.state.backend_str),
            },
        });
    }

    private backendCfgValid(): Boolean {
        try {
            const cfg = JSON.parse(this.state.backend_str);
            for (let key in BackendConfigDefault) {
                if (key === "labels") {
                    if (cfg[key].constructor === Object) {
                        for (let key2 in WeatherLabelsDefault) {
                            if (cfg[key][key2].constructor !== String) {
                                return false;
                            }
                        }
                    } else {
                        return false;
                    }
                } else {
                    if (cfg[key].constructor !== Array) {
                        return false;
                    }
                    for (let id of cfg[key]) {
                        if (isNaN(id)) {
                            return false;
                        }
                    }
                }
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
                        The plugin is currently enabled.
                    </div>
                    <Button
                        className={this.styles.marginTop}
                        variant="destructive"
                        onClick={this.onEnable.bind(this)}
                    >
                        Disable plugin
                    </Button>
                </>
            );
        } else {
            inner = (
                <>
                    <div className={this.styles.colorWeak}>
                        The plugin is currently not enabled.
                    </div>
                    <Button
                        className={this.styles.marginTop}
                        variant="primary"
                        onClick={this.onDisable.bind(this)}
                    >
                        Enable plugin
                    </Button>
                </>
            );
        }

        return <FieldSet label="Enable / Disable">{inner}</FieldSet>;
    }

    public render(): ReactNode {
        const cfg_valid = this.backendCfgValid();

        return (
            <div>
                {this.renderEnableDisable()}

                <FieldSet
                    label="API Settings"
                    className={this.styles.marginTopXl}
                >
                    <Field
                        label="Postgres Datasource"
                        description="An existing empowerd PostgreSQL datasource"
                    >
                        <AsyncSelect
                            loadOptions={this.fetchDatasources}
                            defaultOptions
                            value={this.state.datasource}
                            onChange={this.onChangeDatasource.bind(this)}
                        />
                    </Field>
                    <Field
                        label="API Url"
                        description="URL of the empowerd API"
                        className={this.styles.marginTop}
                    >
                        <Input
                            width={60}
                            label={`API Url`}
                            value={this.state?.apiUrl}
                            placeholder={`E.g.: http://localhost:3001/`}
                            onChange={this.onChangeApiUrl.bind(this)}
                        />
                    </Field>
                    <Field
                        label="Config JSON"
                        description="Empowerd UI configuration JSON"
                        className={this.styles.marginTop}
                    >
                        <Input
                            width={60}
                            label={`Config JSON`}
                            value={this.state?.backend_str}
                            required
                            invalid={!cfg_valid}
                            placeholder={`E.g.: { solars: [1], meters: [2] }`}
                            onChange={this.onChangeConfig.bind(this)}
                        />
                    </Field>
                    <div className={this.styles.marginTop}>
                        <Button
                            type="submit"
                            onClick={this.onSubmit.bind(this)}
                            disabled={Boolean(!this.state.apiUrl || !cfg_valid)}
                        >
                            Save API settings
                        </Button>
                    </div>
                </FieldSet>
            </div>
        );
    }
}
