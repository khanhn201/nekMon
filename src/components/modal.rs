use leptos::html::Dialog;
use leptos::prelude::*;

#[component]
pub fn Modal(opened: RwSignal<bool>, children: ChildrenFn) -> impl IntoView {
    move || {
        opened.get().then(|| {
            let modal_ref = NodeRef::<Dialog>::new();
            Effect::new(move |_| {
                // Show after rendered
                if let Some(dialog) = modal_ref.get() {
                    let _ = dialog.show_modal();
                }
            });
            view! {
                <dialog
                    closedby="any"
                    node_ref=modal_ref
                    class="bg-white rounded top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
                    on:close=move |_| opened.set(false)
                >
                    {children()}
                </dialog>
            }
        })
    }
}

#[component]
pub fn Menu(opened: RwSignal<bool>, children: ChildrenFn) -> impl IntoView {
    move || {
        opened.get().then(|| {
            let modal_ref = NodeRef::<Dialog>::new();
            Effect::new(move |_| {
                // Show after rendered
                if let Some(dialog) = modal_ref.get() {
                    let _ = dialog.show();
                }
            });
            view! {
                <dialog
                    closedby="any"
                    node_ref=modal_ref
                    class="absolute bg-white shadow rounded z-50"
                    on:close=move |_| opened.set(false)
                >
                    <div class="w-max flex flex-col">
                        {children()}
                    </div>
                </dialog>
            }
        })
    }
}
