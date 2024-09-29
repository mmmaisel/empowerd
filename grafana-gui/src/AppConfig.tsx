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

export type BackendConfig = {
    solars: number[];
    generators: number[];
    heatpumps: number[];
};

export const BackendConfigDefault = {
    solars: [],
    generators: [],
    heatpumps: [],
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
};

export class AppConfig extends Component<AppConfigProps, AppConfigState> {
    styles: AppConfigStyles;

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
            backend: BackendConfigDefault,
        };
    }

    getStyles(): AppConfigStyles {
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

    onChangeApiUrl = (event: ChangeEvent<HTMLInputElement>) => {
        this.setState({
            ...this.state,
            apiUrl: event.target.value.trim(),
        });
    };

    onChangeDatasource = (event: SelectableValue<PsqlDatasource>) => {
        this.setState({
            ...this.state,
            datasource: event,
        });
    };

    onEnable = (_event: MouseEvent<HTMLButtonElement>) => {
        this.updatePluginAndReload(this.props.plugin.meta.id, {
            enabled: true,
            pinned: true,
            jsonData: this.props.plugin.meta.jsonData,
        });
    };

    onDisable = (_event: MouseEvent<HTMLButtonElement>) => {
        this.updatePluginAndReload(this.props.plugin.meta.id, {
            enabled: false,
            pinned: false,
            jsonData: this.props.plugin.meta.jsonData,
        });
    };

    onSubmit = (_event: MouseEvent<HTMLButtonElement>) => {
        const { enabled, id, pinned } = this.props.plugin.meta;

        this.updatePluginAndReload(id, {
            enabled,
            pinned,
            jsonData: {
                apiUrl: this.state.apiUrl,
                datasource: this.state.datasource.value,
                backend: this.state.backend,
            },
        });
    };

    updatePluginAndReload = async (
        pluginId: string,
        data: Partial<PluginMeta<ConfigJson>>
    ) => {
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
    };

    updatePlugin = async (pluginId: string, data: Partial<PluginMeta>) => {
        const request = getBackendSrv().fetch({
            url: `/api/plugins/${pluginId}/settings`,
            method: "POST",
            data,
        });
        const response = await lastValueFrom(request);

        return response.data;
    };

    fetchDatasources = async (): Promise<
        Array<SelectableValue<PsqlDatasource>>
    > => {
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
    };

    renderEnableDisable(): ReactNode {
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
                        onClick={this.onEnable}
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
                        onClick={this.onDisable}
                    >
                        Enable plugin
                    </Button>
                </>
            );
        }

        return <FieldSet label="Enable / Disable">{inner}</FieldSet>;
    }

    render(): ReactNode {
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
                            onChange={this.onChangeDatasource}
                        />
                    </Field>
                    <Field
                        label="API Url"
                        description="URL of the empowerd API"
                        className={this.styles.marginTop}
                    >
                        <Input
                            width={60}
                            id="api-url"
                            label={`API Url`}
                            value={this.state?.apiUrl}
                            placeholder={`E.g.: http://localhost:3001/`}
                            onChange={this.onChangeApiUrl}
                        />
                    </Field>
                    <div className={this.styles.marginTop}>
                        <Button
                            type="submit"
                            onClick={this.onSubmit}
                            disabled={Boolean(!this.state.apiUrl)}
                        >
                            Save API settings
                        </Button>
                    </div>
                </FieldSet>
            </div>
        );
    }
}
