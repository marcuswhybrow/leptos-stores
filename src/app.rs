use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_meta::*;
use leptos_router::{
    components::{FlatRoutes, Route, Router},
    StaticSegment,
};
use reactive_stores::{Patch, Store};
use serde::{Deserialize, Serialize};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <link rel="stylesheet" id="leptos" href="/pkg/my_app.css"/>
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico"/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <FlatRoutes fallback=|| "Page not found.">
                <Route path=StaticSegment("") view=Home ssr=leptos_router::SsrMode::InOrder />
            </FlatRoutes>
        </Router>
    }
}

/// It pays to keep sub-components logic simple, espeically when it comes to signal-based reactive
/// systems such as Leptos.
///
/// Each item renders a "Delete" button to remove itself from the Store. One could pass the Store
/// itself to the component and update the Signal there. But this scatters Signal writes across
/// your application, making the reactivity decision tree hard to reason about, leading to
/// convoluted debugging of any emerging errors accessing disposal signals.
///
/// I've found it preferable to only pass read-only state down in components, and callbacks
/// to requests state changes from some some higher level, centralised place.
///
/// Note the `on_delete` argument. It's important **not** to pass signals back up the reactivity
/// scope but instead to use raw values, in this case a u128.
#[component]
fn Item(
    #[prop(into)] item: reactive_stores::Field<Item>,
    on_delete: impl Fn(u128) + Clone + Copy + 'static,
) -> impl IntoView {
    view! {
        <div class="flex gap-2">
            <div class="flex-grow">{ move || format!("{} ({})", item.value().get(), item.id().get()) }</div>
            <button
                class="bg-neutral-200 hover:bg-neutral-300 px-4 py-1 rounded"
                on:click=move |_| {
                    let id = item.id().get_untracked();
                    on_delete(id);
                }
            >Delete</button>
        </div>
    }
}

#[component]
fn Items() -> impl IntoView {
    // In Leptos a Resource defines some code that one would like to begin computing
    // on the server imeediately. By default, everything else is first delivered to
    // the browser, and then evaluated.
    //
    // One can ask leptos to wait for a resource to complete **before** sending any
    // assets to the browser, or to allow both to be sent in parallel, at their own
    // speed, and to be married together later in the browser. Each has it's place.
    //
    // In this instance we are asking for the resource to block the sending of any
    // assets to the browser before evaluation completes by using
    // `Resource::new_blocking`. This improves Search Engine Optimisation, ensuring
    // all data is available upon the first page request. It's also useful for
    // preventing unsightly jumps as data is loaded in fits and starts.
    //
    // If that's not imporant we can instead use `Resource::new` to allow Leptos to
    // use multiple requests to send all non-Resource assets to the browser for a
    // fast initial page load, with separate requests for asynchronously loading
    // Resources's separately in parallel.
    //
    //
    // # Rendering Modes
    //
    // Whenever we use a blocking resource the simplest strategy would be for Leptos
    // to pause the entire response being sent to the browser, to wait all Resources
    // to complete and only then to send the entire response.
    //
    // This is strategy is known as Async Rendering, but forces Leptos to wait for
    // all Resources, even ones that might take a very long time which we'd rather not
    // block wait on (Resource::new invocations).
    //
    // Whenever we might have a mix of Resource:new_blocking and Resource::new, we can
    // use the In Order strategy to ensure all blocking resources block the initial
    // response, whilst non-blocking resources are sent in parallel and married in
    // later via parallel asynchronous requests.
    //
    // There are other stragegies, I'll leave to you can read about here:
    // - https://docs.rs/leptos_router/latest/leptos_router/enum.SsrMode.html
    //
    // In our case we'd like the InOrder strategy, and this is specified for the
    // Home componet's route in the App component above.
    let items_resource =
        Resource::new_blocking(|| (), move |_| async { get_items().await.unwrap() });

    move || {
        let store = Store::new(Data {
            items: items_resource.get().unwrap(),
        });

        // Most pitfalls were encountered deleting a specific item:
        //
        // 1. Read the item ID **before** writing the Vec from which it was reactively derived.
        //
        // ❌ store.items().update(|items| {
        //     let position = items.iter().position(|i| i.id == item.id().get()).unwrap();
        //     items.remove(position);
        // });
        //
        // ✅ let id = item.id().get();
        // store.items().update(|items| {
        //     let position = items.iter().position(|i| i.id == id).unwrap();
        //     items.remove(position);
        // });
        //
        // 2. Item ID must be a copy and not a reference to survive it's disposal by
        // the subsequent Signal write via, say, `store.items().update(...)`.
        //
        // ❌ let id = items.id().read();
        // let id: &u128 = id.as_borrowed();
        // store.items().update(...);
        //
        // ✅ let id = items.id().get();
        // store.items().update(...);
        //
        let on_delete = move |id: u128| {
            store.items().update(|items| {
                let index = items.iter().position(|item| item.id == id).unwrap();
                items.remove(index);
            });
        };

        view! {
            <div class="flex gap-2 mb-4">
                <button
                    class="bg-neutral-200 hover:bg-neutral-300 px-4 py-2 rounded"
                    on:click=move |_| {
                        store.items().update(move |items| {
                            let id = uuid::Uuid::new_v4();
                            items.push(Item {
                                id: id.as_u128(),
                                value: "Value".to_string(),
                            });
                        });
                    }
                >
                    Add
                </button>
                <button
                    class="bg-neutral-200 hover:bg-neutral-300 px-4 py-2 rounded"
                    on:click=move |_| {
                        store.items().update(|items| {
                            let len = items.len();
                            if len >= 2 {
                                let item = items.get_mut(len - 2).unwrap();
                                item.value = "Mutated".to_string();
                            }
                        });
                    }
                >
                    Mutate n-1
                </button>
                <button
                    class="bg-neutral-200 hover:bg-neutral-300 px-4 py-2 rounded"
                    on:click=move |_| {
                        store.items().update(move |items| {
                            if items.len() > 0 {
                                items.remove(0);
                            }
                        });
                    }
                >
                    Delete 0
                </button>
            </div>
            <div class="flex flex-col gap-4">
                <For each=move || store.items() key=|i|i.id().get() let:item>
                    <Item item on_delete />
                </For>
            </div>
        }
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <Title text="Store Vec Demo"/>
        <main class="grid justify-center content-center mt-[20vh]">
            <div class="max-w-xl">
                <p class="mb-16 [&_a]:text-sky-600 [&_a]:font-bold [&_a:hover]:underline">This demo is a reference for how to add, update and delete items from a <a href="https://doc.rust-lang.org/std/vec/struct.Vec.html">Vec</a> inside of a <a href="https://docs.rs/reactive_stores/latest/reactive_stores/struct.Store.html">Store</a> derived from a <a href="https://docs.rs/leptos/latest/leptos/prelude/struct.Resource.html">Resource</a>. { r#"It's"# } suprisingly easy to convolute the reactivity decision tree leading to impenetrable error messages.</p>
                // Suspense component define the boundary of use for any Resource accessed
                // within. Calling `resource.get()` outside of a Suspense throws a warning.
                // Calling `resource.get()` within a suspense can be unwrapped.
                //
                // See the <Items /> component comments for more on Resources.
                <Suspense>
                    <Items />
                </Suspense>
            </div>
        </main>
    }
}

#[derive(Debug, Clone, Store, Patch, Serialize, Deserialize)]
pub struct Item {
    /// An item's ID uniquely identifies each item in a keyed list
    /// such as Leptos' For component. Use an UUID is great way to
    /// generate random and unique ID's without centralised checking.
    pub id: u128,

    /// A demonstration example of some data
    pub value: String,
}

#[derive(Debug, Clone, Store, Patch, Serialize, Deserialize)]
pub struct Data {
    /// It's imperative to use to a Copy type such as u128 for the
    /// Store's Vec key. Using a String, for example, throws an
    /// obtuse error when later iterating over the items in a Leptos
    /// For component.
    #[store(key: u128 = |item| item.id)]
    items: Vec<Item>,
}

#[server]
pub async fn get_items() -> Result<Vec<Item>, ServerFnError> {
    return Ok(vec![
        Item {
            id: uuid::Uuid::new_v4().as_u128(),
            value: "great".to_string(),
        },
        Item {
            id: uuid::Uuid::new_v4().as_u128(),
            value: "amasing".to_string(),
        },
    ]);
}
