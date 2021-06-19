import {install_panic_hook, answer_to_life, show_alert, do_panic} from 'examplelib';

install_panic_hook();

interface Exports {
    alert(): void
    panic(): void
}

declare var window: {wasm: Exports};

window.wasm = {
    alert() {
        show_alert(answer_to_life().toString());
    },
    panic() {
        do_panic();
    },
};
