import { ComponentCustomProperties } from 'vue'
import { Store } from 'vuex'

declare module '@vue/runtime-core' {
    interface ComponentCustomProperties {
        $store: Store<FlightCoreStore>,
        $t: (key: string, ...params: any[]) => string;
    }
}
