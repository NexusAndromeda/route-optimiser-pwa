use yew::prelude::*;
use web_sys::HtmlInputElement;
use crate::models::Company;

#[derive(Properties, PartialEq)]
pub struct CompanySelectorProps {
    pub show: bool,
    pub companies: Vec<Company>,
    pub on_close: Callback<()>,
    pub on_select: Callback<Company>,
    pub loading: bool,
}

#[function_component(CompanySelector)]
pub fn company_selector(props: &CompanySelectorProps) -> Html {
    let search_ref = use_node_ref();
    let search_term = use_state(|| String::new());
    
    let on_search = {
        let search_ref = search_ref.clone();
        let search_term = search_term.clone();
        
        Callback::from(move |_: KeyboardEvent| {
            if let Some(input) = search_ref.cast::<HtmlInputElement>() {
                search_term.set(input.value());
            }
        })
    };
    
    let filtered_companies: Vec<Company> = if search_term.is_empty() {
        props.companies.clone()
    } else {
        let term = search_term.to_lowercase();
        props.companies
            .iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&term) || 
                c.code.to_lowercase().contains(&term)
            })
            .cloned()
            .collect()
    };
    
    let modal_class = if props.show {
        "company-modal show"
    } else {
        "company-modal"
    };
    
    html! {
        <div class={modal_class}>
            <div class="company-modal-content">
                <div class="company-modal-header">
                    <h3>{"Seleccionar Empresa"}</h3>
                    <button
                        class="btn-close"
                        onclick={props.on_close.reform(|_| ())}
                    >
                        {"✕"}
                    </button>
                </div>
                
                <div class="company-search">
                    <input
                        type="text"
                        id="company-search"
                        placeholder="Buscar empresa..."
                        ref={search_ref}
                        onkeyup={on_search}
                    />
                </div>
                
                <div class="company-list">
                    if props.loading {
                        <div class="company-loading">{"⏳ Cargando empresas..."}</div>
                    } else if filtered_companies.is_empty() {
                        <div class="company-empty">{"No se encontraron empresas"}</div>
                    } else {
                        { for filtered_companies.iter().map(|company| {
                            let company_clone = company.clone();
                            let on_select = props.on_select.clone();
                            
                            html! {
                                <div
                                    class="company-item"
                                    onclick={Callback::from(move |_| {
                                        on_select.emit(company_clone.clone());
                                    })}
                                >
                                    <div class="company-name">{&company.name}</div>
                                    <div class="company-code">{&company.code}</div>
                                </div>
                            }
                        }) }
                    }
                </div>
            </div>
        </div>
    }
}

