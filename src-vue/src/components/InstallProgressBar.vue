<script lang="ts">
import { defineComponent } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { InstallProgress } from '../../../src-tauri/bindings/InstallProgress';

export default defineComponent({
    name: 'InstallProgressBar',
    computed: {
        progressBarStyle(): string {
            return !this.install_or_update ? 'hide-progress' : '';
        }
    },
    data() {
        return {
            percentage: 0,
            color: '#409EFF',
            install_or_update: false,
            status: "unknown",
            current_downloaded: -1,
            total_size: -1,
        };
    },
    methods: {
        formatBytes(bytes: number, decimals = 2) {
            if (bytes === 0) return '0 Bytes';
            const k = 1000;
            const dm = decimals < 0 ? 0 : decimals;
            const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
        },
        formatText() {
            if (this.status == "Downloading") {
                const current_downloaded_string = this.formatBytes(this.current_downloaded);
                const total_size_string = this.formatBytes(this.total_size);
                const status = this.$t("generic.downloading");
                return `${status}: ${current_downloaded_string}/${total_size_string}`;
            }
            if (this.status == "Extracting") {
                return this.$t("generic.extracting");
            }
            return "Inactive";  // Needed to keep same size format when progress bar is hidden
        }
    },
    mounted() {
        getCurrentWindow().listen<InstallProgress>(
            'northstar-install-download-progress',
            ({ event, payload }) => {
                event
                this.install_or_update = true;
                let progress = payload;
                this.status = progress.state;
                if (progress.state == "Downloading") {
                    this.percentage = ((Number(progress.current_downloaded) / Number(progress.total_size)) * 100);
                    this.color = '#409EFF';
                    this.current_downloaded = Number(progress.current_downloaded);
                    this.total_size = Number(progress.total_size);
                }
                if (progress.state == "Extracting") {
                    this.percentage = 100;
                    this.color = '#67C23A';
                }
                if (progress.state == "Done") {
                    // Clear state again
                    this.install_or_update = false
                }
            }
        );
    }
});
</script>

<template>
    <el-progress
        :class="progressBarStyle"
        :format="formatText"
        :percentage="percentage"
        :color="color"
        :indeterminate="status === 'Extracting'"
        :duration="1"
    >
    </el-progress>
</template>

<style scoped>
.el-progress {
    margin-top: 10px;
}

/* Set progress bar width */
.el-progress:deep(.el-progress-bar) {
    width: 200px;
    flex-grow: initial;
}

.el-progress:deep(.el-progress__text) {
    line-height: 1.2;
}

.hide-progress {
    opacity: 0;
}
</style>
