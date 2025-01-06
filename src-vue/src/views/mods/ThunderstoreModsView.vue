<template>
    <div class="fc-container" style="padding: 0">
        <div v-if="mods.length === 0" class="fc__changelog__container">
            <el-progress :show-text="false" :percentage="50" :indeterminate="true" />
        </div>

        <!-- Message displayed if no mod matched searched words -->
        <div v-else-if="filteredMods.length === 0" class="noModMessage">
            {{ $t('mods.online.no_match') }}<br/>
            {{ $t('mods.online.try_another_search') }}
        </div>

        <el-scrollbar v-else class="container" ref="scrollbar">
            <div class="card-container">
                <div class="pagination_container" v-if="shouldDisplayPagination">
                    <el-pagination
                        :currentPage="currentPageIndex + 1"
                        layout="prev, pager, next"
                        :page-size="modsPerPage"
                        :total="modsList.length"
                        @update:current-page="onPaginationChange"
                    />
                </div>

                <!-- Mod cards -->
                <thunderstore-mod-card v-for="mod of currentPageMods" v-bind:key="mod.name" :mod="mod" />
            </div>

            <!-- Bottom pagination -->
            <div class="card-container">
                <div class="pagination_container">
                    <el-pagination
                        class="fc_bottom__pagination"
                        v-if="shouldDisplayPagination"
                        :currentPage="currentPageIndex + 1"
                        layout="prev, pager, next"
                        :page-size="modsPerPage"
                        :total="modsList.length"
                        @update:current-page="onPaginationChange"
                        @current-change="scrollTop"
                    />
                </div>
            </div>
        </el-scrollbar>
    </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { ThunderstoreMod } from "../../../../src-tauri/bindings/ThunderstoreMod";
import ThunderstoreModCard from "../../components/ThunderstoreModCard.vue";
import { ScrollbarInstance } from "element-plus";
import { SortOptions } from "../../utils/SortOptions.d";
import { ThunderstoreModVersion } from "../../../../src-tauri/bindings/ThunderstoreModVersion";
import { fuzzy_filter } from "../../utils/filter";
import { isThunderstoreModOutdated } from "../../utils/thunderstore/version";


export default defineComponent({
    name: "ThunderstoreModsView",
    components: { ThunderstoreModCard },
    async mounted() {
        this.$store.commit('fetchThunderstoreMods');
    },
    computed: {
        showDeprecatedMods(): boolean {
            return this.$store.state.search.showDeprecatedMods;
        },
        showNsfwMods(): boolean {
            return this.$store.state.search.showNsfwMods;
        },
        searchValue(): string {
            return this.$store.getters.searchWords;
        },
        selectedCategories(): Object[] {
            return this.$store.state.search.selectedCategories;
        },
        modSorting(): SortOptions {
            return Object.values(SortOptions)[Object.keys(SortOptions).indexOf(this.$store.state.search.sortValue)];
        },
        mods(): ThunderstoreMod[] {
            return this.$store.state.thunderstoreMods;
        },
        filteredMods(): ThunderstoreMod[] {
            if (this.searchValue.length === 0 && this.selectedCategories.length === 0) {
                return this.mods;
            }

            return this.mods.filter((mod: ThunderstoreMod) => {
                // Filter with search words (only if search field isn't empty)
                const inputMatches: boolean = this.searchValue.length === 0
                    || (
                        fuzzy_filter(mod.name, this.searchValue) ||
                        fuzzy_filter(mod.owner, this.searchValue) ||
                        mod.versions[0].description.toLowerCase().includes(this.searchValue)
                    );

                // Filter out deprecated mods
                const showDeprecated = !mod.is_deprecated || this.showDeprecatedMods;

                // Filter out NSFW mods
                const showNsfw = !mod.has_nsfw_content || this.showNsfwMods;

                // Filter with categories (only if some categories are selected)
                const categoriesMatch: boolean = this.selectedCategories.length === 0
                    || mod.categories
                        .filter((category: string) => this.selectedCategories.includes(category))
                        .length === this.selectedCategories.length;

                return inputMatches && categoriesMatch && showDeprecated && showNsfw;
            });
        },
        modsList(): ThunderstoreMod[] {
            // Use filtered mods if user is searching, vanilla list otherwise.
            const mods: ThunderstoreMod[] = this.searchValue.length !== 0 || this.selectedCategories.length !== 0
                ? this.filteredMods
                : this.mods
                    .filter(mod => this.showDeprecatedMods || !mod.is_deprecated)
                    .filter(mod => this.showNsfwMods || !mod.has_nsfw_content);

            // Sort mods regarding user selected algorithm.
            let compare: (a: ThunderstoreMod, b: ThunderstoreMod) => number;
            switch (this.modSorting) {
                case SortOptions.NAME_ASC:
                    compare = (a: ThunderstoreMod, b: ThunderstoreMod) => a.name.localeCompare(b.name);
                    break;
                case SortOptions.NAME_DESC:
                    compare = (a: ThunderstoreMod, b: ThunderstoreMod) => -1 * a.name.localeCompare(b.name);
                    break;
                case SortOptions.DATE_ASC:
                    compare = (a: ThunderstoreMod, b: ThunderstoreMod) => a.date_updated.localeCompare(b.date_updated);
                    break;
                case SortOptions.DATE_DESC:
                    compare = (a: ThunderstoreMod, b: ThunderstoreMod) => -1 * a.date_updated.localeCompare(b.date_updated);
                    break;
                case SortOptions.MOST_DOWNLOADED:
                    compare = (a: ThunderstoreMod, b: ThunderstoreMod) => {
                        const aTotal = a.versions.reduce((prev, next) => {
                            return { downloads: prev.downloads + next.downloads } as ThunderstoreModVersion;
                        }).downloads;
                        const bTotal = b.versions.reduce((prev, next) => {
                            return { downloads: prev.downloads + next.downloads } as ThunderstoreModVersion;
                        }).downloads;
                        return -1 * (aTotal - bTotal);
                    };
                    break;
                case SortOptions.TOP_RATED:
                    compare = (a: ThunderstoreMod, b: ThunderstoreMod) => -1 * (a.rating_score - b.rating_score);
                    break;
                default:
                    throw new Error('Unknown mod sorting.');
            }

            // Always display outdated mods first
            // (regardless of actual sort order)
            const sortedMods = mods.sort(compare);
            return sortedMods.sort((a, b) => {
                if (isThunderstoreModOutdated(a)) {
                    return -1;
                } else if (isThunderstoreModOutdated(b)) {
                    return 1;
                } else {
                    return compare(a, b);
                }
            })
        },
        modsPerPage(): number {
            return parseInt(this.$store.state.mods_per_page);
        },
        currentPageMods(): ThunderstoreMod[] {
            // User might want to display all mods on one page.
            const perPageValue = this.modsPerPage != 0 ? this.modsPerPage : this.modsList.length;

            const startIndex = this.currentPageIndex * perPageValue;
            const endIndexCandidate = startIndex + perPageValue;
            const endIndex = endIndexCandidate > this.modsList.length ? this.modsList.length : endIndexCandidate;
            return this.modsList.slice(startIndex, endIndex);
        },
        shouldDisplayPagination(): boolean {
            return this.modsPerPage != 0 && this.modsList.length > this.modsPerPage;
        }
    },
    data() {
        return {
            modsBeingInstalled: [] as string[],
            currentPageIndex: 0
        };
    },
    methods: {
        /**
         * This updates current pagination and scrolls view to the top.
         */
        onPaginationChange(index: number) {
            this.currentPageIndex = index - 1;
        },
        scrollTop() {
            setTimeout(() => {
                (this.$refs.scrollbar as ScrollbarInstance).scrollTo({ top: 0, behavior: 'smooth' });
            }, 100)
        }
    },
    watch: {
        searchValue(_: string, __: string) {
            if (this.currentPageIndex !== 0) {
                this.currentPageIndex = 0;
            }
        },
        selectedCategories(_: string[], __: string[]) {
            if (this.currentPageIndex !== 0) {
                this.currentPageIndex = 0;
            }
        }
    }
});
</script>

<style scoped>
.fc__changelog__container {
    padding: 20px 30px;
}

.fc-container:deep(.el-scrollbar__view) {
    padding-left: 0;
    padding-right: 0;
}

.el-timeline-item__timestamp {
    color: white !important;
    user-select: none !important;
}

.search {
    display: inline-block;
    margin: 0 0 0 10px !important;
}

.card-container {
    margin: 0 auto;
}

.pagination_container {
    margin: 5px auto;
    padding: 0 5px;
    max-width: 1000px;
    justify-content: center;
    display: flex;
}

.el-pagination {
    margin: 0;
}

.fc_bottom__pagination {
    padding-bottom: 20px !important;
    padding-right: 10px;
}

/* Card container dynamic size */

.card-container {
    --thunderstore-mod-card-width: 178px;
    --thunderstore-mod-card-margin: 5px;
    --thunderstore-mod-card-columns-count: 1;

    width: calc(var(--thunderstore-mod-card-width) * var(--thunderstore-mod-card-columns-count) + var(--thunderstore-mod-card-margin) * 2 * var(--thunderstore-mod-card-columns-count));
}

@media (min-width: 628px) {
    .card-container {
        --thunderstore-mod-card-columns-count: 2;
    }
}

@media (min-width: 836px) {
    .card-container {
        --thunderstore-mod-card-columns-count: 3;
    }
}

@media (min-width: 1006px) {
    .card-container {
        --thunderstore-mod-card-columns-count: 4;
    }
}

@media (min-width: 1196px) {
    .card-container {
        --thunderstore-mod-card-columns-count: 5;
    }
}

@media (min-width: 1386px) {
    .card-container {
        --thunderstore-mod-card-columns-count: 6;
    }
}

@media (min-width: 1576px) {
    .card-container {
        --thunderstore-mod-card-columns-count: 7;
    }
}

@media (min-width: 1766px) {
    .card-container {
        --thunderstore-mod-card-columns-count: 8;
    }
}
</style>
