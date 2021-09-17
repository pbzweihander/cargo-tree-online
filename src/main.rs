use std::collections::BTreeSet as Set;

use anyhow::{Context as _, Error};
use cargo_lock::dependency::graph::{EdgeDirection, Graph, NodeIndex};
use cargo_lock::dependency::tree::Tree;
use cargo_lock::Lockfile;
use petgraph::visit::EdgeRef;
use yew::prelude::*;

enum Msg {
    Submit,
}

struct App {
    link: ComponentLink<Self>,
    lock_file_input_ref: NodeRef,
    dependency_tree: Option<Result<Tree, Error>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            lock_file_input_ref: NodeRef::default(),
            dependency_tree: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Submit => {
                self.dependency_tree = Some(parse_lock_file_from_input(&self.lock_file_input_ref));
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <textarea
                    ref={self.lock_file_input_ref.clone()}
                    placeholder="Paste Cargo.lock file here"
                    spellcheck="false"
                    style="display: block; width: 500px; min-height: 220px"
                />
                <button
                    style="display: block; width: 100px; height: 30px"
                    onclick=self.link.callback(|_| Msg::Submit)
                >
                    { "Submit" }
                </button>
                {
                    match &self.dependency_tree {
                        Some(Ok(tree)) => render_tree(tree),
                        Some(Err(error)) => html! { <div>{ format!("{}", error) }</div> },
                        None => html! { <></> },
                    }
                }
            </div>
        }
    }
}

fn parse_lock_file_from_input(input_ref: &NodeRef) -> Result<Tree, Error> {
    let input = input_ref
        .cast::<web_sys::HtmlInputElement>()
        .context("Invalid node ref state")?;
    let lock_file = toml::from_str::<Lockfile>(&input.value())?;
    Ok(lock_file.dependency_tree()?)
}

fn render_tree(tree: &Tree) -> Html {
    let mut visited = Set::new();
    let mut levels_continue = Vec::new();
    let graph = tree.graph();
    html! {
        <ul>
            {
                for tree.roots()
                    .into_iter()
                    .map(|node_index| render_node(&mut visited, &mut levels_continue, graph, node_index))
            }
        </ul>
    }
}

fn render_node(
    visited: &mut Set<NodeIndex>,
    levels_continue: &mut Vec<bool>,
    graph: &Graph,
    node_index: NodeIndex,
) -> Html {
    let package = &graph[node_index];
    let new = visited.insert(node_index);

    if !new {
        return html! {
            <li>{ format!("{} {}", &package.name, &package.version) }</li>
        };
    }

    let dependencies = graph
        .edges_directed(node_index, EdgeDirection::Outgoing)
        .map(|edge| edge.target())
        .collect::<Vec<_>>();

    let dependencies = dependencies.iter().enumerate().map(|(i, dependency)| {
        levels_continue.push(i < (dependencies.len() - 1));
        let node = render_node(visited, levels_continue, graph, *dependency);
        levels_continue.pop();
        node
    });

    html! {
        <li>
            { format!("{} {}", &package.name, &package.version) }
            <ul>{ for dependencies }</ul>
        </li>
    }
}

fn main() {
    yew::start_app::<App>();
}
