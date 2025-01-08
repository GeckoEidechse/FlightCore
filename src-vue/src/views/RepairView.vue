<template>
    <div class="fc-container">
        <el-scrollbar>
            <el-alert :title="$t('generic.informationShort')" type="info" :closable="false" show-icon>
                {{ $t('settings.repair.window.warning') }}
            </el-alert>

            <h1>{{ $t('settings.repair.title') }}</h1>

            <h2>Northstar</h2>

            <el-button type="primary" @click="disableAllModsButCore">
                {{ $t('settings.repair.window.disable_all_but_core') }}
            </el-button>

            <el-button type="primary" @click="forceInstallNorthstar">
                {{ $t('settings.repair.window.force_reinstall_ns') }}
            </el-button>

            <el-button type="primary" @click="killNorthstar">
                {{ $t('settings.repair.window.kill_northstar_process') }}
            </el-button>

            <el-button type="primary" @click="disableModsettingsMod">
                {{ $t('settings.repair.window.disable_modsettings') }}
            </el-button>

            <h2>FlightCore</h2>

            <el-button type="primary" @click="cleanUpDownloadFolder">
                {{ $t('settings.repair.window.force_delete_temp_dl') }}
            </el-button>

            <el-button type="primary" @click="clearFlightCorePersistentStore">
                {{ $t('settings.repair.window.delete_persistent_store') }}
            </el-button>
        </el-scrollbar>
    </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { GameInstall } from "../utils/GameInstall";
import { InstallProgress } from "../../../src-tauri/bindings/InstallProgress";
import { invoke } from "@tauri-apps/api";
import { ReleaseCanal } from "../utils/ReleaseCanal";
import { Store } from 'tauri-plugin-store-api';
import { showErrorNotification, showNotification } from "../utils/ui";
import { appWindow } from "@tauri-apps/api/window";
const persistentStore = new Store('flight-core-settings.json');

export default defineComponent({
    name: "RepairView",
    computed: {
        lang(): string {
            return this.$root!.$i18n.locale;
        }
    },
    methods: {
        async disableAllModsButCore() {
            await invoke("disable_all_but_core", { gameInstall: this.$store.state.game_install })
                .then((message) => {
                    showNotification(this.$t('generic.success'), this.$t('settings.repair.window.disable_all_but_core_success'));
                })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async forceInstallNorthstar() {
            // Send notification telling the user to wait for the process to finish
            const notification = showNotification(
                this.$t('settings.repair.window.reinstall_title'),
                this.$t('settings.repair.window.reinstall_text'),
                'info',
                0
            );

            let install_northstar_result = invoke("install_northstar_wrapper", { gameInstall: this.$store.state.game_install, northstarPackageName: ReleaseCanal.RELEASE });

            appWindow.listen<InstallProgress>(
                'northstar-install-download-progress',
                ({ event, payload }) => {
                    let typed_payload = payload;
                    console.log("current_downloaded:", typed_payload.current_downloaded);
                    console.log("total_size:        ", typed_payload.total_size);
                    console.log("state:             ", typed_payload.state);
                }
            );
            await install_northstar_result
                .then((message) => {
                    // Send notification
                    showNotification(this.$t('generic.done'), this.$t('settings.repair.window.reinstall_success'));
                    this.$store.commit('checkNorthstarUpdates');
                })
                .catch((error) => {
                    showErrorNotification(error);
                    console.error(error);
                })
                .finally(() => {
                    // Clear old notification
                    notification.close();
                });
        },
        async cleanUpDownloadFolder() {
            await invoke("clean_up_download_folder_wrapper", { gameInstall: this.$store.state.game_install, force: true }).then((message) => {
                // Show user notification if task completed.
                showNotification(this.$t('generic.done'), this.$t('generic.done'));
            })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async clearFlightCorePersistentStore() {
            // Clear store...
            await persistentStore.clear();
            // ...and save
            await persistentStore.save();
        },
        async disableModsettingsMod() {
            await invoke("set_mod_enabled_status", { gameInstall: this.$store.state.game_install, modName: "Mod Settings", modVersion: "", isEnabled: false })
                .then((message) => {
                    showNotification(this.$t('generic.success'), this.$t('settings.repair.window.disable_modsettings_success'));
                })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async killNorthstar() {
            await invoke("kill_northstar")
                .then((message) => {
                    // Just a visual indicator that it worked
                    showNotification('Success');
                })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
    },
    watch: {
        // Lang value is propagated to repair view after it's mounted, so we need to watch
        // its value, and update window title accordingly.
        lang(newv: string) {
            appWindow.setTitle(this.$t('settings.repair.window.title'));
        }
    }
});
</script>

<style scoped>
.fc-container {
    padding-top: 0px;
}
</style>
