use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BalModalProps {
    pub on_close: Callback<()>,
    pub on_select: Callback<bool>,
}

#[function_component(BalModal)]
pub fn bal_modal(props: &BalModalProps) -> Html {
    let close = props.on_close.clone();
    let close_overlay = props.on_close.clone();
    
    html! {
        <div class="modal active">
            <div class="modal-overlay" onclick={Callback::from(move |_| close_overlay.emit(()))}></div>
            <div class="modal-content modal-small" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                <div class="modal-header">
                    <h2>{"Accès boîte aux lettres"}</h2>
                    <button class="btn-close" onclick={Callback::from(move |_| close.emit(()))}>
                        {"✕"}
                    </button>
                </div>
                <div class="modal-body">
                    <p class="question-text">
                        {"Y a-t-il un accès à la boîte aux lettres ?"}
                    </p>
                    <div class="choice-buttons">
                        <button
                            class="btn-choice btn-yes"
                            onclick={{
                                let on_select = props.on_select.clone();
                                Callback::from(move |_| on_select.emit(true))
                            }}
                        >
                            {"✅ Oui"}
                        </button>
                        <button
                            class="btn-choice btn-no"
                            onclick={{
                                let on_select = props.on_select.clone();
                                Callback::from(move |_| on_select.emit(false))
                            }}
                        >
                            {"❌ Non"}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

