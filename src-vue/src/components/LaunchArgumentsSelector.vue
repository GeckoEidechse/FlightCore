<template>
    <div>
        <div :class="containerClasses">
            <div class="fc-launch_arg_tag_container"
                    v-for="(argument, index) in arguments">
                <!-- Official arguments -->
                <el-tooltip
                    v-if="argument.i18nEntry.length !== 0"
                    class="box-item"
                    :content="$t(argument.i18nEntry)"
                    placement="bottom"
                >
                    <el-check-tag
                        :checked="values[index]"
                        @change="onChange(index)"
                    >
                        {{ argument.argumentName }}
                    </el-check-tag>
                </el-tooltip>

                <!-- Custom arguments -->
                <el-tag
                    v-else
                    closable
                    disable-transitions
                    @close="onClose(index, argument.argumentName)"
                >
                    {{ argument.argumentName }}
                </el-tag>
            </div>

            <!-- User-input tag -->
            <div class="fc-launch_arg_tag_container">
                <el-input
                    v-if="inputVisible"
                    ref="InputRef"
                    class="fc-tag__input"
                    v-model="inputValue"
                    @keyup.enter="handleInputConfirm"
                    @blur="handleInputConfirm"
                />
                <el-button v-else class="button-new-tag ml-1 fc-tag__input" @click="showInput">
                    {{ $t('settings.launch_args.new_arg_btn') }}
                </el-button>
            </div>
        </div>


        <!-- Language selector -->
        <div :class="containerClasses">
            <el-select v-if="displayLanguageSelector"
                v-model="langArgumentValue"
                class="m-2 fc-launch_arg_tag_container fc-tag__input"
                :placeholder="$t('settings.launch_args.select_game_language')"
                @change="onLanguageSelection"
            >
                <el-option
                    v-for="item in langArgumentOptions"
                    :key="item.value"
                    :label="item.label"
                    :value="item.value"
                />
            </el-select>
        </div>
    </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { LaunchArgument } from '../utils/LaunchArgument';
import { NorthstarState } from '../utils/NorthstarState';
import {invoke} from "@tauri-apps/api";
import { showErrorNotification } from '../utils/ui';
import { Store } from 'tauri-plugin-store-api';
import {argumentsStoreKey} from "../main";

const persistentStore = new Store('flight-core-settings.json');

export default defineComponent({
    name: 'LaunchArgumentsSelector',
    computed: {
        arguments(): LaunchArgument[] {
            return (this.localCustomArgs.concat(this.officialArguments))
                .sort((a, b) => a.argumentName.localeCompare(b.argumentName));
        },
        displayLanguageSelector(): boolean {
            const langArgPrefix = '-language=';
            return this.arguments
                .map(arg => arg.argumentName)
                .filter(name => name.substring(0, langArgPrefix.length) === langArgPrefix)
                .length === 0;
        },
        officialArguments(): LaunchArgument[] {
            return [
                new LaunchArgument("-disablelogs", "settings.launch_args.descriptions.disablelogs"),
                new LaunchArgument("-vanilla", "settings.launch_args.descriptions.vanilla"),
                new LaunchArgument("-northstar", "settings.launch_args.descriptions.northstar"),
                new LaunchArgument("-dedicated", "settings.launch_args.descriptions.dedicated"),
                new LaunchArgument("-waitfordebugger", "settings.launch_args.descriptions.waitfordebugger"),
                new LaunchArgument("-enablechathooks", "settings.launch_args.descriptions.enablechathooks"),
                new LaunchArgument("-noplugins", "settings.launch_args.descriptions.noplugins"),
                new LaunchArgument("-novid", "settings.launch_args.descriptions.novid"),
                new LaunchArgument("-nosound", "settings.launch_args.descriptions.nosound")
            ];
        },
        containerClasses(): string {
            return this.gamePathIsSelected ? 'fc-tags_container' : 'fc-tags_container disabled_container';
        },
        gamePathIsSelected(): boolean {
            return this.$store.state.northstar_state !== NorthstarState.GAME_NOT_FOUND;
        },
        langArgumentOptions() {
            const languages = ['english', 'french', 'german', 'italian', 'japanese', 'mspanish', 'portuguese', 'russian', 'spanish', 'tchinese'];
            return languages.map(lang => ({value: lang, label: lang}));
        }
    },
    data: () => ({
        inputValue: '',
        inputVisible: false,

        langArgumentValue: '',
        values: [] as boolean[],
        localCustomArgs: [] as LaunchArgument[]
    }),
    methods: {
        onLanguageSelection( lang: string ) {
            this.createNewArgument( `-language=${lang}` );
            this.langArgumentValue = '';
        },
        createNewArgument(arg: string) {
            let allArgumentsNames: string[] = this.arguments.map(arg => arg.argumentName);
            if (allArgumentsNames.indexOf(arg) !== -1) {
                console.warn(`Argument "${arg}" already present, ignoring.`);
            } else {
                const newArgument: LaunchArgument = new LaunchArgument(arg);
                this.localCustomArgs.push( newArgument );
                allArgumentsNames = this.arguments.map(arg => arg.argumentName);

                const index: number = allArgumentsNames.indexOf(newArgument.argumentName);
                this.values.splice(index, 0, true);
                this.saveLaunchArgumentsToFile();
            }
        },
        onChange(index: number) {
            this.values[index] = !this.values[index];
            this.saveLaunchArgumentsToFile();
        },
        onClose(index: number, argumentName: string) {
            // remove item state value
            this.values.splice(index, 1);

            // remove item from list of custom arguments
            const localIndex = this.localCustomArgs.map(arg => arg.argumentName).indexOf(argumentName);
            if (localIndex === -1) {
                console.error(`Failed removing argument "${argumentName}".`);
                return;
            }
            this.localCustomArgs.splice(localIndex, 1);

            this.saveLaunchArgumentsToFile();
        },
        saveLaunchArgumentsToFile() {
            const newArgs = this.arguments
                .filter((value: LaunchArgument, index: number) => {
                    return this.values[index];
                })
                .map((value: LaunchArgument) => value.argumentName);

            persistentStore.set(argumentsStoreKey, newArgs)
                .then(async () => {
                    await persistentStore.save();
                    console.log("Launch arguments updated.");
                })
                .catch((err: any) => {
                    showErrorNotification(err);
                });
        },
        showInput() {
            this.inputVisible = true;
            this.$nextTick(() => {
                // @ts-ignore
                this.$refs.InputRef.input.focus();
            });
        },
        handleInputConfirm() {
            if (this.inputValue.length !== 0) {
                this.createNewArgument(this.inputValue);
            }
            this.inputVisible = false;
            this.inputValue = '';
        },
    },
    async mounted() {
        this.values = this.arguments.map(a => false);

        const fileArgs: string[] | null = await persistentStore.get(argumentsStoreKey);
        if (fileArgs === null) {
            console.log('No local launch arguments to load, exiting.');
            return;
        }

        // Only add to local arguments those who are not in official arguments array
        this.localCustomArgs = fileArgs
            .filter(arg => this.officialArguments.map(oArg => oArg.argumentName).indexOf(arg) === -1)
            .map(arg => new LaunchArgument(arg));

        this.arguments.forEach((argument, index) => {
            if (fileArgs.includes(argument.argumentName)) {
                this.values[index] = true;
            }
        });
    }
});
</script>

<style scoped>

.fc-launch_arg_tag_container {
    display: inline-block;
    margin: 0 8px 8px 8px;
}

.fc-tags_container {
    transform: translateX(-8px);
}

.fc-tags_container .el-select {
    min-width: 222px;
}

.el-tag {
    background-color: var(--el-color-primary-light-8);
    color: var(--el-color-primary);
    font-size: var(--el-font-size-base);
    line-height: var(--el-font-size-base);
    padding: 14px 5px 12px 15px;
    transition: var(--el-transition-all);
    font-weight: 700;
}

.el-check-tag {
    padding: 9px 15px 5px 15px;
    transition: none;
}

.disabled_container {
    pointer-events: none;
    filter: grayscale(1);
}

.fc-tag__input {
    width: auto;
    height: 28px;
    --el-component-size: 28px;
}
</style>
