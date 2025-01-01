import { AppPlugin } from "@grafana/data";
import { App } from "./App";
import { AppConfig } from "./AppConfig";

export const plugin = new AppPlugin<{}>().setRootPage(App).addConfigPage({
    title: "Configuration",
    icon: "cog",
    body: AppConfig,
    id: "configuration",
});
