use futures_util::StreamExt;
use dioxus::prelude::*;
use dioxus_signals::ReactiveContext;

pub fn use_branched_signal<T: 'static>(mut f: impl FnMut() -> T + 'static) -> Signal<T> {
    let location = std::panic::Location::caller();

    use_hook(|| {
        let (rc, mut changed) = ReactiveContext::new_with_origin(location);

        let value = rc.run_in(&mut f);
        let mut result = Signal::new(value);

        spawn_isomorphic(async move {
            while changed.next().await.is_some() {
                // Remove any pending updates
                while changed.try_next().is_ok() {}
                let new_value = rc.run_in(&mut f);
                result.set(new_value)
            }
        });
        result
    })
}

