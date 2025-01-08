<template>
    <el-card shadow="hover">
        <div class="name">
            {{ mod.name }} <span v-if="mod.version != null">(v{{ mod.version }})</span>
            <img
                v-if="mod.thunderstore_mod_string != null"
                :title="$t('mods.local.part_of_ts_mod') + '\n' + mod.thunderstore_mod_string"
                src="/src/assets/thunderstore-icon.png"
                class="image"
                height="16"
            />
        </div>
        <div>
            <el-switch 
                style="--el-switch-on-color: #13ce66; --el-switch-off-color: #8957e5"
                v-model="mod.enabled"
                :before-change="() => updateWhichModsEnabled(mod)"
                :loading="global_load_indicator"
                class="switch"
            />
            <el-popconfirm
                :title="$t('mods.local.delete_confirm')"
                :confirm-button-text="$t('generic.yes')"
                :cancel-button-text="$t('generic.no')"
                @confirm="deleteMod(mod)"
            >
                <template #reference>
                    <el-button type="danger">
                        {{ $t('mods.local.delete') }}
                    </el-button>
                </template>
            </el-popconfirm>
        </div>
    </el-card>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { invoke } from "@tauri-apps/api";
import { NorthstarMod } from "../../../src-tauri/bindings/NorthstarMod";
import { showErrorNotification, showNotification } from "../utils/ui";

export default defineComponent({
    name: "LocalModCard",
    props: {
        mod: {
            required: true,
            type: Object as () => NorthstarMod
        }
    },
    data() {
        return {
            global_load_indicator: false,
        };
    },
    methods: {
        async updateWhichModsEnabled(mod: NorthstarMod) {
            this.global_load_indicator = true;

            // enable/disable specific mod
            try {
                await invoke("set_mod_enabled_status", {
                    gameInstall: this.$store.state.game_install,
                    modName: mod.name,
                    modVersion: mod.version,
                    // Need to set it to the opposite of current state,
                    // as current state is only updated after command is run
                    isEnabled: !mod.enabled,
                })
            }
            catch (error) {
                showErrorNotification(`${error}`);
                this.global_load_indicator = false;
                return false;
            }

            this.global_load_indicator = false;
            return true;
        },
        async deleteMod(mod: NorthstarMod) {
            await invoke("delete_northstar_mod", { gameInstall: this.$store.state.game_install, nsmodName: mod.name })
                .then((message) => {
                    // Just a visual indicator that it worked
                    showNotification(this.$t('mods.local.success_deleting', { modName: mod.name }));
                })
                .catch((error) => {
                    showErrorNotification(error);
                })
                .finally(() => {
                    this.$store.commit('loadInstalledMods');
                });
        },
    }
});
</script>

<style scoped>
    /*
        This is a hack to style the card body 
        since it doesn't work with scoped styles
    */
    :deep(.el-card__body) {
        display: flex !important;
        align-items: center;
        width: 100%;
        justify-content: space-between;
    }

    .name {
        display: flex;
    }

    .image {
        margin: 0 5px;
    }

    .switch {
        padding-left: 5px;
        padding-right: 5px;
    }
</style>
