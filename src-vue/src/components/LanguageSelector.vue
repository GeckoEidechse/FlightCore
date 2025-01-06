<template>
    <el-select v-model="value" class="m-2" 
        :placeholder="$t('settings.language_select')" size="large"
        @change="onChange"
    >
        <el-option
            v-for="item in options"
            :key="item.value"
            :label="item.label"
            :value="item.value"
        />
  </el-select>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { load } from '@tauri-apps/plugin-store';
const persistentStore = await load('flight-core-settings.json', { autoSave: false });

export default defineComponent({
    name: 'LanguageSelector',
    data: () => ({
        value: '',
        options: [
            {
                value: 'en',
                label: 'English'
            },
            {
                value: 'fr',
                label: 'Français'
            },
            {
                value: 'de',
                label: 'Deutsch'
            },
            {
                value: 'es',
                label: 'Español'
            },
            {
                value: 'pl',
                label: 'polski'
            },
            {
                value: 'ru',
                label: 'русский'
            },
            {
                value: 'it',
                label: 'Italiano'
            },
            {
                value: 'da',
                label: 'Dansk'
            },
            {
                value: 'zh_Hans',
                label: '简体中文'
            },
        ]
    }),
    mounted: async function () {
        const lang: string = await persistentStore.get('lang') as string;
        this.value = lang;
    },
    methods: {
        async onChange(value: string) {
            this.$root!.$i18n.locale = value;
            persistentStore.set('lang', value);
            await persistentStore.save();
        }
    }
})
</script>
