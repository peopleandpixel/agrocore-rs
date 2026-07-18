use icondata::*;
use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Info,
    Warning,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastMessage {
    pub id: Uuid,
    pub message: String,
    pub toast_type: ToastType,
}

#[derive(Copy, Clone)]
pub struct ToastContext {
    pub toasts: ReadSignal<Vec<ToastMessage>>,
    pub add_toast: Callback<(String, ToastType)>,
    pub remove_toast: Callback<Uuid>,
}

pub fn provide_toast_context() {
    let (toasts, set_toasts) = signal(Vec::<ToastMessage>::new());

    let add_toast = Callback::new(move |(message, toast_type): (String, ToastType)| {
        let id = Uuid::new_v4();
        set_toasts.update(|t| {
            t.push(ToastMessage {
                id,
                message,
                toast_type,
            })
        });

        // Auto-remove after 5 seconds
        leptos::task::spawn_local(async move {
            leptos::task::tick().await;
            gloo_timers::future::TimeoutFuture::new(5000).await;
            set_toasts.update(|t| t.retain(|m| m.id != id));
        });
    });

    let remove_toast = Callback::new(move |id: Uuid| {
        set_toasts.update(|t| t.retain(|m| m.id != id));
    });

    provide_context(ToastContext {
        toasts,
        add_toast,
        remove_toast,
    });
}

#[component]
pub fn ToastContainer() -> impl IntoView {
    let context = use_context::<ToastContext>().expect("ToastContext not provided");

    view! {
        <div class="toast-container">
            <For
                each=move || context.toasts.get()
                key=|toast| toast.id
                children=move |toast| {
                    let id = toast.id;
                    let remove = context.remove_toast;
                    let (bg_class, icon) = match toast.toast_type {
                        ToastType::Success => ("bg-success text-success-content", LuCircleCheck),
                        ToastType::Error => ("bg-error text-error-content", LuCircleAlert),
                        ToastType::Info => ("bg-info text-info-content", LuInfo),
                        ToastType::Warning => ("bg-warning text-warning-content", LuTriangleAlert),
                    };

                    view! {
                        <div class=format!("toast-item {} border border-white/10", bg_class)>
                            <Icon icon=icon width="20" height="20" />
                            <span class="flex-1 text-sm font-medium">{toast.message}</span>
                            <button class="btn btn-ghost btn-xs btn-circle" on:click=move |_| remove.run(id)>
                                <Icon icon=LuX width="16" height="16" />
                            </button>
                        </div>
                    }
                }
            />
        </div>
    }
}
