<template>
    <el-dialog
        v-model="showProfileDialog"
        :title="$t('settings.profile.dialog.title')"
        width="75%"
    >
        <el-table :data="availableProfiles" >
            <el-table-column prop="name" label="Name" />
            <el-table-column align="right">
              <template #default="scope">
                <el-popconfirm
                    v-if="scope.row.name != 'R2Northstar'"
                    :title="$t('settings.profile.dialog.delete_confirm')"
                    :confirm-button-text="$t('generic.yes')"
                    :cancel-button-text="$t('generic.no')"
                    @confirm="deleteProfile(scope.row.name)"
                >
                    <template #reference>
                        <el-button type="danger">
                            {{ $t('settings.profile.dialog.delete') }}
                        </el-button>
                    </template>
                </el-popconfirm>
                <el-button @click="cloneProfileDialog(scope.row.name)">
                    {{ $t('settings.profile.dialog.clone') }}
                </el-button>
              </template>
          </el-table-column>
        </el-table>
    </el-dialog>

    <div class="fc-container">
        <el-scrollbar>
            <div class="fc_settings__container">
                <!-- Game folder location -->
                <div class="fc_parameter__panel">
                    <h3>{{ $t('settings.manage_install') }}</h3>
                    <el-input
                        v-model="$store.state.game_install.game_path"
                        :placeholder="$t('settings.choose_folder')"
                        @click="updateGamePath"
                    >
                        <template #prepend>
                            <el-button icon="Folder" @click="updateGamePath"/>
                        </template>
                        <template #append>
                            <el-button @click="openGameInstallFolder">
                                {{ $t('settings.open_game_folder') }}
                            </el-button>
                        </template>
                    </el-input>
                </div>

                <!-- Northstar Active Profile -->
                <div class="fc_parameter__panel" v-if="$store.state.developer_mode">
                    <h3>{{ $t('settings.profile.active') }}</h3>
                    <el-dropdown trigger="click" :disabled="!availableProfiles.length">
                        <el-button>
                            {{ activeProfile }} <el-icon class="el-icon--right" v-if="availableProfiles.length"><arrow-down /></el-icon>
                        </el-button>
                        <template #dropdown>
                            <el-dropdown-menu>
                                <el-dropdown-item v-for="profile in $store.state.available_profiles" @click="switchProfile(profile)">{{ profile }}</el-dropdown-item>
                                <el-dropdown-item divided @click="showProfileDialog = true">{{ $t('settings.profile.edit') }}</el-dropdown-item>
                            </el-dropdown-menu>
                        </template>
                    </el-dropdown>
                </div>

                <!-- Launch arguments selection -->
                <div class="fc_parameter__panel">
                    <h3>{{ $t('settings.launch_args.title') }}</h3>
                    <launch-arguments-selector/>
                </div>

                <!-- Thunderstore mods per page configuration -->
                <div class="fc_parameter__panel">
                    <h3>{{ $t('settings.nb_ts_mods_per_page') }}</h3>
                    <h6>
                        {{ $t('settings.nb_ts_mods_per_page_desc1') }}<br>
                        {{ $t('settings.nb_ts_mods_per_page_desc2') }}
                    </h6>
                    <el-input
                        v-model="modsPerPage"
                        type="number"
                    >
                        <template #append>
                            <el-button @click="modsPerPage = 20">
                                {{ $t('settings.nb_ts_mods_reset') }}
                            </el-button>
                        </template>
                    </el-input>
                </div>

                <div class="fc_parameter__panel">
                    <h3>{{ $t('settings.show_deprecated_mods') }}</h3>
                    <h6>
                        {{ $t('settings.show_deprecated_mods_desc1') }}<br/>
                        {{ $t('settings.show_deprecated_mods_desc2') }}
                    </h6>
                    <span>
                        {{ $t('settings.show_deprecated_mods') }}
                        <el-switch v-model="showDeprecatedMods"></el-switch>
                    </span>
                </div>

                <!-- Interface localization -->
                <div class="fc_parameter__panel">
                    <h3>{{ $t('settings.language') }}</h3>
                    <language-selector/>
                </div>

                <!-- Repair window -->
                <div class="fc_parameter__panel">
                    <h3>{{ $t('settings.repair.title') }}</h3>
                    <el-button type="primary" @click="openRepairWindow">
                        {{ $t('settings.repair.open_window') }}
                    </el-button>
                </div>

                <!-- About section -->
                <div class="fc_parameter__panel">
                    <h3>{{ $t('settings.about') }}</h3>
                    <div class="fc_northstar__version" @click="activateDeveloperMode">
                        {{ $t('settings.flightcore_version') }} {{ flightcoreVersion === '' ? 'Unknown version' : `${flightcoreVersion}` }}
                    </div>
                </div>

                <!-- Testing section -->
                <div class="fc_parameter__panel">
                    <h3>{{ $t('settings.testing') }}</h3>
                    <span>
                        {{ $t('settings.enable_test_channels') }}
                        <el-switch v-model="enableReleasesSwitch"></el-switch>
                    </span>
                </div>
            </div>
        </el-scrollbar>
    </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { invoke } from "@tauri-apps/api";
import { ReleaseCanal } from "../utils/ReleaseCanal";
import { Store } from 'tauri-plugin-store-api';
import { showErrorNotification, showNotification } from "../utils/ui";
import LanguageSelector from "../components/LanguageSelector.vue";
import LaunchArgumentsSelector from "../components/LaunchArgumentsSelector.vue";
const persistentStore = new Store('flight-core-settings.json');
import { open } from '@tauri-apps/api/shell';
import { i18n } from '../main';
import { ElMessageBox } from 'element-plus'

export default defineComponent({
    name: "SettingsView",
    components: {
        LanguageSelector,
        LaunchArgumentsSelector
    },
    data() {
        return {
            developerModeClicks: 0,
            showProfileDialog: false,
        }
    },
    computed: {
        showDeprecatedMods: {
            get(): boolean {
                return this.$store.state.search.showDeprecatedMods;
            },
            set(value: boolean) {
                this.$store.state.search.showDeprecatedMods = value;
            }
        },
        flightcoreVersion(): string {
            return this.$store.state.flightcore_version;
        },
        enableReleasesSwitch: {
            get(): boolean {
                return this.$store.state.enableReleasesSwitch;
            },
            async set(value: boolean): Promise<void> {
                this.$store.state.enableReleasesSwitch = value;
                persistentStore.set('northstar-releases-switching', { value });
                await persistentStore.save(); // explicit save to disk

                // When disabling switch, we switch release canal to stable release, to avoid users being
                // stuck with release candidate after disabling release switching.
                if (!value && this.$store.state.northstar_release_canal !== ReleaseCanal.RELEASE) {
                    this.$store.commit('toggleReleaseCandidate');
                }
            }
        },
        modsPerPage: {
            get(): number {
                return this.$store.state.mods_per_page;
            },
            async set(value: number) {
                this.$store.state.mods_per_page = value;
                persistentStore.set('thunderstore-mods-per-page', { value });
                await persistentStore.save(); // explicit save to disk
            }
        },
        activeProfile(): String {
            return this.$store.state.game_install.profile || "None";
        },
        availableProfiles(): Object[] {
            let profiles = this.$store.state.available_profiles

            // convert string array to object array so we can fill a table
            let data = profiles.reduce(
                (a: Object[], v: string) => [...a, {"name": v}],
                []
            );

            return data;
        }
    },
    methods: {
        activateDeveloperMode() {
            this.developerModeClicks += 1;
            if (this.developerModeClicks >= 6 && !this.$store.state.developer_mode) {
                this.$store.commit('toggleDeveloperMode');
                showNotification(
                    this.$t('settings.dev_mode_enabled_title'),
                    this.$t('settings.dev_mode_enabled_text'),
                    'info'
                );
                this.developerModeClicks = 0;
            }
        },
        async updateGamePath() {
            this.$store.commit('updateGamePath');
        },
        async openRepairWindow() {
            await invoke("open_repair_window")
                .then((message) => { })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async openGameInstallFolder() {
            // Verify the game path is actually set
            if (!this.$store.state.game_install.game_path) {
                showErrorNotification(
                    i18n.global.tc('notification.game_folder.not_found.text'),
                    i18n.global.tc('notification.game_folder.not_found.title')
                );
                return;
            }

            // Opens the folder in default file explorer application
            await open(`${this.$store.state.game_install.game_path}`);
        },
        async switchProfile(value: string) {
            let store = this.$store;
            let state = store.state;

            await invoke("validate_profile", { gameInstall: state.game_install, profile: value })
                .then(async (message) => {
                    if (!message)
                    {
                        // Profile is no longer valid, inform the user...
                        showErrorNotification(
                            i18n.global.tc('notification.profile.invalid.text'),
                            i18n.global.tc('notification.profile.invalid.title')
                        );

                        // ...and refresh
                        store.commit('fetchProfiles');
                        return;
                    }

                    state.game_install.profile = value;

                    // Check for Northstar updates
                    store.commit('checkNorthstarUpdates');

                    // Save change in persistent store
                    await persistentStore.set('game-install', { value: state.game_install });
                    await persistentStore.save(); // explicit save to disk
                })
                .catch((error) => {
                    console.error(error);
                    showErrorNotification(error);
                });
        },
        async cloneProfileDialog(profile: string) {
            ElMessageBox.prompt(
                i18n.global.tc("settings.profile.dialog.new_profile_name"),
                i18n.global.tc("settings.profile.dialog.title"),
                {
                    confirmButtonText: i18n.global.tc("generic.confirm"),
                    cancelButtonText: i18n.global.tc("generic.cancel"),
                }
            ).then(async ({ value }) => {
                await this.cloneProfile(profile, value);
            }).catch(() => {
                // Nothing to do here
            })
        },
        async cloneProfile(old_profile: string, new_profile: string) {
            let store = this.$store;
            await invoke("clone_profile", {
                gameInstall: store.state.game_install,
                oldProfile: old_profile,
                newProfile: new_profile
            }).then(async (message) => {
                store.commit('fetchProfiles');
                showNotification('Success');
            }).catch((error) => {
                console.error(error);
                showErrorNotification(error);
            });
        },
        async deleteProfile(profile: string) {
            let store = this.$store;
            await invoke("delete_profile", {
                gameInstall: store.state.game_install,
                profile: profile,
            }).then(async (message) => {
                if (profile == store.state.game_install.profile)
                {
                    // trying to delete the active profile, lets switch to the default profile
                    await this.switchProfile("R2Northstar");
                }
                store.commit('fetchProfiles');
                showNotification('Success');
            }).catch((error) => {
                console.error(error);
                showErrorNotification(error);
            });
        },
    },
    mounted() {
        document.querySelector('input')!.disabled = true;
        this.$store.commit('fetchProfiles');
    },
    unmounted() {
        if (('' + this.modsPerPage) === '') {
            console.warn('Incorrect value for modsPerPage, resetting it to 20.');
            this.modsPerPage = 20;
        }
    }
});
</script>

<style scoped>
.fc_settings__container {
    max-width: 1200px;
    margin: 0 auto;
}

h3:first-of-type {
    margin-top: 0;
    margin-bottom: 1em;
    text-transform: uppercase;
    font-weight: unset;
}

.el-input,
.el-select {
    width: 50%;
}

.el-switch {
    margin-left: 50px;
}


/* Parameter panel styles */
.fc_parameter__panel {
    margin-bottom: 30px;
}

.fc_parameter__panel h3 {
    margin-bottom: 5px;
}

.fc_parameter__panel h6 {
    margin-top: 0;
    margin-bottom: 12px;
}
</style>
