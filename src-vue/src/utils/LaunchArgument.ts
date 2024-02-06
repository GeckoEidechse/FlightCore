export class LaunchArgument {
    public argumentName: string;
    public i18nEntry: string;

    constructor(argumentName: string, i18nEntry: string = '') {
        this.argumentName = argumentName;
        this.i18nEntry = i18nEntry;
    }
}