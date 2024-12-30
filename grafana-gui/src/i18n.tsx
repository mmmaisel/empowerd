import i18next, { t } from "i18next";

const resources = {
    de: {
        translation: {
            "enable-disable": "Aktivieren / Deaktivieren",
            "enable-plugin": "Plugin aktivieren",
            "disable-plugin": "Plugin deaktivieren",
            "currently-disabled": "Das Plugin ist aktuell nicht aktiviert.",
            "currently-enabled": "Das Plugin ist aktuell aktiviert.",
            "api-settings": "API Einstellungen",
            "psql-source": "Postgres Datenquelle",
            "psql-source-desc": "Eine bestehende PostgreSQL Datenquelle.",
            "api-location": "API Pfad",
            "api-location-desc":
                "Pfad zur empowerd API. Muss sich auf dem gleichen Server " +
                "wie Grafana befinden.",
            eg: "z.B.",
            "config-json": "Konfigurations JSON",
            "config-json-desc": "Empowerd UI Konfigurations-String.",
            "save-settings": "Einstellungen speichern",
        },
    },
    en: {
        translation: {
            "enable-disable": "Enable / Disable",
            "enable-plugin": "Enable plugin",
            "disable-plugin": "Disable plugin",
            "currently-disabled": "The plugin is currently not enabled.",
            "currently-enabled": "The plugin is currently enabled.",
            "api-settings": "API Settings",
            "psql-source": "Postgres Datasource",
            "psql-source-desc": "An existing empowerd PostgreSQL datasource.",
            "api-location": "API Location",
            "api-location-desc":
                "Path to the empowerd API. Must be on the same server " +
                "as Grafana.",
            eg: "e.g.",
            "config-json": "Config JSON",
            "config-json-desc": "Empowerd UI configuration string.",
            "save-settings": "Save settings",
        },
    },
};

export { t };

export function init_i18n() {
    i18next.init({
        lng: navigator.language,
        fallbackLng: "en",
        supportedLngs: ["de", "en"],
        initAsync: false,
        resources,
    });
}
