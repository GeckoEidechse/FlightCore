<template>
    <div class="fc-container">
        <el-scrollbar>
            <el-alert title="Warning" type="warning" :closable="false" show-icon>
                This page is designed for developers. Some of the buttons here can break your Northstar install if you do not know what you're doing!
            </el-alert>

            <h3>Basic:</h3>

            <el-button type="primary" @click="disableDevMode">
                Disable developer mode
            </el-button>

            <el-button type="primary" @click="crashApplication">
                Panic button
            </el-button>

            <h3>Linux:</h3>

            <el-button type="primary" @click="installNSProton">
                Install NSProton
            </el-button>

            <el-button type="primary" @click="uninstallNSProton">
                Remove NSProton
            </el-button>

            <el-button type="primary" @click="getLocalNSProtonVersion">
                Get local NSProton Version
            </el-button>

            <h3>Testing:</h3>

            <el-button type="primary" @click="launchGameWithoutChecks">
                Launch Northstar (bypass all checks)
            </el-button>

            <el-button type="primary" @click="launchGameViaSteam">
                Launch Northstar via Steam
            </el-button>

            <el-button type="primary" @click="installLauncherGitMain">
                Install launcher from main branch
            </el-button>

            <br />
            <br />

            <el-button type="primary" @click="getAvailableNorthstarVersions">
                Get available versions
            </el-button>

            <el-select v-model="selected_ns_version" class="m-2" placeholder="Versions">
                <el-option
                    v-for="item in ns_versions"
                    :key="item.value"
                    :label="item.label"
                    :value="item"
                />
            </el-select>

            <el-button type="primary" @click="installNorthstarVersion">
                Install
            </el-button>

            <h3>Repair:</h3>

            <el-button type="primary" @click="checkCgnat">
                Run tracert and collect hop count
            </el-button>

            <el-button type="primary" @click="getInstalledMods">
                Get installed mods
            </el-button>

            <h3>Testing</h3>
            <pull-requests-selector />

            <h3>Mod install:</h3>

            <el-input v-model="mod_to_install_field_string" placeholder="Please input Thunderstore dependency string (example: AuthorName-ModName-1.2.3)" clearable />

            <el-button type="primary" @click="installMod">
                Install mod
            </el-button>

            <h3>Release management</h3>
            <el-select v-model="selected_project" placeholder="Select">
            <el-option
                v-for="item in project"
                :key="item.value"
                :label="item.label"
                :value="item.value"
                />
            </el-select>
            <el-button type="primary" @click="getTags">
                Get tags
            </el-button>

            <el-select v-model="firstTag" class="m-2" placeholder="First tag">
                <el-option
                    v-for="item in ns_release_tags"
                    :key="item.value"
                    :label="item.label"
                    :value="item"
                />
            </el-select>
            <el-select v-model="secondTag" class="m-2" placeholder="Second tag">
                <el-option
                    v-for="item in ns_release_tags"
                    :key="item.value"
                    :label="item.label"
                    :value="item"
                />
            </el-select>

            <el-button type="primary" @click="compareTags">
                Compare Tags
            </el-button>

            <el-button type="primary" @click="copyReleaseNotesToClipboard">
                Copy to clipboard
            </el-button>

            <el-input
                v-model="release_notes_text"
                type="textarea"
                :rows="5"
                placeholder="Output"
            />

            <h3>Release announcements</h3>

            <el-button type="primary" @click="generateReleaseAnnouncementMessage">
                Generate release announcement
            </el-button>

            <el-input
                v-model="discord_release_announcement_text"
                type="textarea"
                :rows="5"
                placeholder="Output"
            />
        </el-scrollbar>
    </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NorthstarLaunchOptions } from "../../../src-tauri/bindings/NorthstarLaunchOptions";
import { TagWrapper } from "../../../src-tauri/bindings/TagWrapper";
import { NorthstarThunderstoreReleaseWrapper } from "../../../src-tauri/bindings/NorthstarThunderstoreReleaseWrapper";
import PullRequestsSelector from "../components/PullRequestsSelector.vue";
import { showErrorNotification, showNotification } from "../utils/ui";

export default defineComponent({
    name: "DeveloperView",
    components: {
        PullRequestsSelector
    },
    data() {
        return {
            mod_to_install_field_string: "",
            release_notes_text: "",
            discord_release_announcement_text: "",
            first_tag: { label: '', value: { name: '' } },
            second_tag: { label: '', value: { name: '' } },
            ns_release_tags: [] as TagWrapper[],
            ns_versions: [] as NorthstarThunderstoreReleaseWrapper[],
            selected_ns_version: { label: '', value: { package: '', version: '' } } as NorthstarThunderstoreReleaseWrapper,
            selected_project: "FlightCore",
            project: [
                {
                    value: 'FlightCore',
                    label: 'FlightCore',
                },
                {
                    value: 'Northstar',
                    label: 'Northstar',
                }
            ],
        }
    },
    computed: {
        firstTag: {
            get(): TagWrapper {
                return this.first_tag;
            },
            set(value: TagWrapper) {
                this.first_tag = value;
            }
        },
        secondTag: {
            get(): TagWrapper {
                return this.second_tag;
            },
            set(value: TagWrapper) {
                this.second_tag = value;
            }
        },
    },
    methods: {
        disableDevMode() {
            this.$store.commit('toggleDeveloperMode');
        },
        async crashApplication() {
            await invoke("force_panic");
            showErrorNotification("Never should have been able to get here!");
        },
        async launchGameWithoutChecks() {
            let launch_options: NorthstarLaunchOptions = { bypass_checks: true, launch_via_steam: false };
            this.$store.commit('launchGame', launch_options);
        },
        async launchGameViaSteam() {
            let launch_options: NorthstarLaunchOptions = { bypass_checks: false, launch_via_steam: true };
            this.$store.commit('launchGameSteam', launch_options);
        },
        async getInstalledMods() {
            await invoke("get_installed_mods_and_properties", { gameInstall: this.$store.state.game_install }).then((message) => {
                // Simply console logging for now
                // In the future we should display the installed mods somewhere
                console.log(message);

                // Just a visual indicator that it worked
                showNotification('Success');
            })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async installMod() {
            let mod_to_install = this.mod_to_install_field_string;
            await invoke<string>("install_mod_wrapper", { gameInstall: this.$store.state.game_install, thunderstoreModString: mod_to_install }).then((message) => {
                // Show user notification if mod install completed.
                showNotification(`Installed ${mod_to_install}`, message);
            })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async getTags() {
            await invoke<TagWrapper[]>("get_list_of_tags", { project: this.selected_project })
                .then((message) => {
                    this.ns_release_tags = message;
                    showNotification("Done", "Fetched tags");
                    this.first_tag = this.ns_release_tags[1];
                    this.second_tag = this.ns_release_tags[0];
                    this.compareTags();
                })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async compareTags() {
            await invoke<string>("compare_tags", { project: this.selected_project, firstTag: this.firstTag.value, secondTag: this.secondTag.value })
                .then((message) => {
                    this.release_notes_text = message;
                    showNotification("Done", "Generated release notes");
                    this.copyReleaseNotesToClipboard();
                })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async installLauncherGitMain() {

            const notification = showNotification(`Installing git main`, 'Please wait', 'info', 0);

            await invoke<string>("install_git_main", { gameInstallPath: this.$store.state.game_install.game_path })
                .then((message) => {
                    this.release_notes_text = message;
                    showNotification("Done", `Installed launcher build from ${message}`);
                })
                .catch((error) => {
                    showErrorNotification(error);
                })
                .finally(() => {
                    // Clear old notification
                    notification.close();
                });
        },
        async getAvailableNorthstarVersions() {
            await invoke<NorthstarThunderstoreReleaseWrapper[]>("get_available_northstar_versions")
                .then((message) => {
                    this.ns_versions = message;
                    showNotification("Done", "Fetched all available Northstar versions");
                })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async installNorthstarVersion() {
            // Send notification telling the user to wait for the process to finish
            const notification = showNotification(
                `Installing Northstar version v${this.selected_ns_version.value.version}`,
                "Please wait",
                'info',
                0
            );

            let install_northstar_result = invoke("install_northstar_wrapper", { gameInstall: this.$store.state.game_install, northstarPackageName: this.selected_ns_version.value.package, versionNumber: this.selected_ns_version.value.version });

            await install_northstar_result
                .then(() => {
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
        async installNSProton() {
            showNotification(`Started NSProton install`);
            await invoke("install_northstar_proton_wrapper")
                .then(() => { showNotification(`Done`); })
                .catch((error) => { showNotification(`Error`, error, "error"); })
        },
        async uninstallNSProton() {
            await invoke("uninstall_northstar_proton_wrapper")
                .then(() => { showNotification(`Done`); })
                .catch((error) => { showNotification(`Error`, error, "error"); })
        },
        async getLocalNSProtonVersion() {
            await invoke("get_local_northstar_proton_wrapper_version")
                .then((message) => { showNotification(`NSProton Version`, message as string); })
                .catch((error) => { showNotification(`Error`, error, "error"); })
        },
        async checkCgnat() {
            await invoke<string>("check_cgnat")
                .then((message) => {
                    showNotification(message);
                })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async copyReleaseNotesToClipboard() {
            navigator.clipboard.writeText(this.release_notes_text)
                .then(() => {
                    showNotification("Copied to clipboard");
                })
                .catch(() => {
                    showErrorNotification("Failed copying to clipboard");
                });
        },
        async generateReleaseAnnouncementMessage() {
            await invoke<string>("generate_release_note_announcement", { })
                .then((message) => {
                    this.discord_release_announcement_text = message;
                    showNotification("Done", "Generated announcement");
                })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
    }
});
</script>

<style scoped>
</style>
