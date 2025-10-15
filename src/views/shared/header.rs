use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub show_settings: bool,
    pub on_toggle_settings: Callback<MouseEvent>,
}

#[function_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    let onclick = {
        let callback = props.on_toggle_settings.clone();
        Callback::from(move |e: MouseEvent| callback.emit(e))
    };
    
    html! {
        <header class="app-header">
            <h1>{"Route Optimizer"}</h1>
            <button class="btn-settings" onclick={onclick}>
                {"⚙️"}
            </button>
        </header>
    }
}

