import { createApp } from 'vue'
import { createI18n } from "vue-i18n";
import App from './App.vue'
import ElementPlus from "element-plus";
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import { store } from './plugins/store';
import PlayView from "./views/PlayView.vue";
import ChangelogView from "./views/ChangelogView.vue";
import ModsView from "./views/ModsView.vue";
import SettingsView from "./views/SettingsView.vue";
import DeveloperView from "./views/DeveloperView.vue";
import RepairView from "./views/RepairView.vue";
import {createRouter, createWebHashHistory} from "vue-router";
import en from "./i18n/lang/en.json";
import fr from "./i18n/lang/fr.json";
import da from "./i18n/lang/da.json";
import de from "./i18n/lang/de.json";
import es from "./i18n/lang/es.json";
import pl from "./i18n/lang/pl.json";
import ru from "./i18n/lang/ru.json";
import it from "./i18n/lang/it.json";
import zh_Hans from "./i18n/lang/zh_Hans.json";


const app = createApp(App);

// internationalization
export const i18n = createI18n({
    locale: 'en',
    fallbackLocale: 'en',
    messages: {
        en, fr, da, de, es, pl, ru, it, zh_Hans
    }
});
app.use(i18n);

// styles
import 'element-plus/theme-chalk/index.css';
import './style.css'

app.use(ElementPlus);

// icons
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
    app.component(key, component);
}

// style
app.use( store, '$store' );


// routes
const routes = [
    { path: '/', name: 'Main', component: async () => PlayView},
    { path: '/changelog', name: 'Changelog', component: async () => ChangelogView},
    { path: '/mods', name: 'Mods', component: async () => ModsView},
    { path: '/settings', name: 'Settings', component: async () => SettingsView},
    { path: '/dev', name: 'Dev', component: async () => DeveloperView},
    { path: '/repair', name: 'Repair', component: async () => RepairView},
];
export const router = createRouter({
    history: createWebHashHistory(),
    routes, // short for `routes: routes`
});
app.use(router);

// Store keys
export const argumentsStoreKey = 'launch_arguments';

app.mount('#app')
