import { invoke,  } from "@tauri-apps/api";
import { PullsApiResponseElement } from "../../../../src-tauri/bindings/PullsApiResponseElement";
import { PullRequestType } from '../../../../src-tauri/bindings/PullRequestType';
import { store } from "../store";
import { showErrorNotification, showNotification } from "../../utils/ui";
import shell from "@tauri-apps/plugin-shell"

interface PullRequestStoreState {
    searchValue: string,
    pull_requests_launcher: PullsApiResponseElement[],
    pull_requests_mods: PullsApiResponseElement[],
}

export const pullRequestModule = {
    state: () => ({
        pull_requests_launcher: [],
        pull_requests_mods: [],
    }),
    mutations: {
        async getPullRequests(state: PullRequestStoreState, pull_request_type: PullRequestType) {
            await invoke<PullsApiResponseElement[]>("get_pull_requests_wrapper", { installType: pull_request_type })
                .then((message) => {
                    switch (pull_request_type) {
                        case "Mods":
                            state.pull_requests_mods = message;
                            break;

                        case "Launcher":
                            state.pull_requests_launcher = message;
                            break;

                        default:
                            console.error("We should never end up here");
                    }
                })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async downloadLauncherPR(state: PullRequestStoreState, pull_request: PullsApiResponseElement) {
            await invoke<string>("get_launcher_download_link", { commitSha: pull_request.head.sha })
                .then((url) => {
                    // Open URL in default HTTPS handler (i.e. default browser)
                    shell.open(url);
                })
                .catch((error) => {
                    showErrorNotification(error);
                });
        },
        async downloadModsPR(state: PullRequestStoreState, pull_request: PullsApiResponseElement) {
            let url = `https://github.com/${pull_request.head.repo.full_name}/archive/refs/heads/${pull_request.head.ref}.zip`
            shell.open(url);
        },
        async installLauncherPR(state: PullRequestStoreState, pull_request: PullsApiResponseElement) {
            // Send notification telling the user to wait for the process to finish
            const notification = showNotification(`Installing launcher PR ${pull_request.number}`, 'Please wait', 'info', 0);

            await invoke("apply_launcher_pr", { pullRequest: pull_request, gameInstall: store.state.game_install })
                .then((message) => {
                    console.log(message);
                    // Show user notification if mod install completed.
                    showNotification(`Done`, `Installed ${pull_request.number}: "${pull_request.title}"`);
                })
                .catch((error) => {
                    showErrorNotification(error);
                })
                .finally(() => {
                    // Clear old notification
                    notification.close();
                });
        },
        async installModsPR(state: PullRequestStoreState, pull_request: PullsApiResponseElement) {
            // Send notification telling the user to wait for the process to finish
            const notification = showNotification(`Installing mods PR ${pull_request.number}`, 'Please wait', 'info', 0);

            await invoke("apply_mods_pr", { pullRequest: pull_request, gameInstall: store.state.game_install })
                .then((message) => {
                    // Show user notification if mod install completed.
                    showNotification(
                        `Done`,
                        `Installed ${pull_request.number}: "${pull_request.title}"\nMake sure to launch via batch file or by specifying correct profile!`,
                        'success',
                        7000
                    );
                })
                .catch((error) => {
                    showErrorNotification(error);
                })
                .finally(() => {
                    // Clear old notification
                    notification.close();
                });
        },
    }
}
