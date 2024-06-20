use std::cell::RefCell;
use std::rc::Rc;
use ratatui::text::Text;
use std::collections::HashMap;
use std::collections::HashSet;
use crate::app_modes::viewport::{UseViewport, Viewport as AppViewport};
use crate::app_modes::{input, AppMode, BaseMode, Drawable};
use crate::config::Color as ConfigColor;
use crate::config::TermvizConfig;
use crate::config::{ImageListenerConfig, ListenerConfig, ListenerConfigColor, PoseListenerConfig};
use rand::Rng;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation};
use tui::{Frame, Viewport};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Clone)]
struct SelectableTopics {
    // `items` is the state managed by your application.
    items: Vec<TreeItem<'static, String>>,
    state: TreeState<&'static str>,
    // `state` is the state that can be modified by the UI. It stores the index of the selected
    // item as well as the offset computed during the previous draw call (used to implement
    // natural scrolling).
}

impl SelectableTopics {
    fn new() -> SelectableTopics {
        SelectableTopics {
            items: vec![TreeItem::new_leaf("a".to_string(), "Alfa".to_string())],
            state: TreeState::default(),
        }
    }

    // Select the next item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    // pub fn next(&mut self) {
    //     let i = match self.state.selected() {
    //         Some(i) => {
    //             if i >= self.items.len() - 1 {
    //                 0
    //             } else {
    //                 i + 1
    //             }
    //         }
    //         None => 0,
    //     };
    //     self.state.select(Some(i));
    // }

    // // Select the previous item. This will not be reflected until the widget is drawn in the
    // // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    // pub fn previous(&mut self) {
    //     let i = match self.state.selected() {
    //         Some(i) => {
    //             if i == 0 {
    //                 self.items.len() - 1
    //             } else {
    //                 i - 1
    //             }
    //         }
    //         None => 0,
    //     };
    //     self.state.select(Some(i));
    // }

    // pub fn add(&mut self, element: [String; 2]) {
    //     self.items.push(element);
    // }

    // // Default to 0 if none is selected, the handling of empty vectors should be
    // // handled by the caller
    // pub fn pop(&mut self) -> [String; 2] {
    //     let i = match self.state.selected() {
    //         Some(i) => {
    //             if i > self.items.len() - 1 {
    //                 self.items.len() - 1
    //             } else {
    //                 i
    //             }
    //         }
    //         None => 0,
    //     };
    //     self.items.remove(i)
    // }
}

pub struct TopicManager {
    // Topic Manger loads the active and supported topics into two lists.
    // The User can shift elements between available and selected topics.
    // topics can only be present in on of the lists.
    frames: Vec<Vec<String>>,
    tf_frames: SelectableTopics,
    config: TermvizConfig,
    was_saved: bool,
    viewport: Rc<RefCell<AppViewport>>,
}

impl TopicManager {
    pub fn new(
        viewport: Rc<RefCell<AppViewport>>,
        config: TermvizConfig, 
    ) -> TopicManager {
        let config = config.clone();

        let base_link_pose = viewport.borrow().tf_listener.lookup_transform(

            &viewport.borrow().static_frame,
            &viewport.borrow().robot_frame,
            rosrust::Time::new(),
        );

        // Fill the state manager with active and supported topics
        TopicManager {
            frames: vec![
                vec![
                    String::from("Test1"),
                    String::from("Test2")
                ]
            ],
            tf_frames: SelectableTopics::new(),
            config: config,
            was_saved: false,
            viewport: viewport,
        }
    }

    

    pub fn save(&mut self) {
        let mut config = self.config.clone();

        // Flush all to get a new config
        config.laser_topics.clear();
        config.marker_array_topics.clear();
        config.marker_topics.clear();
        config.pose_stamped_topics.clear();
        config.pose_array_topics.clear();
        config.path_topics.clear();
        config.polygon_stamped_topics.clear();

        // Store and exit termviz
        let _ = confy::store("termviz", "termviz", &(config));
        self.was_saved = true
    }
}

#[derive(Debug)]
struct TreeNode {
    name: String,
    children: Vec<Box<TreeNode>>,
}

impl TreeNode {
    fn new(name: &str) -> Self {
        TreeNode {
            name: name.to_string(),
            children: Vec::new(),
        }
    }
}

impl<B: Backend> BaseMode<B> for TopicManager {}

impl AppMode for TopicManager {
    fn run(&mut self) {
        let buffered_tfs = self.viewport
            .borrow()
            .tf_listener
            .buffer
            .read()
            .unwrap()
            .transform_data
            .clone()
        ;

        let mut tree: HashMap<String, TreeItem<String>> = HashMap::new();
        let mut parent_map = self.viewport.borrow().tf_listener.buffer.read().unwrap().child_transform_index_no_cycle.clone();
        self.frames.clear();
        self.tf_frames.items.clear();

        // Root nodes are nodes that are not anyone's children
        let mut root_nodes: Vec<Vec<String>> = vec![];
        let mut all_children: HashSet<String> = HashSet::new();

        // Collect all children nodes
        for children in parent_map.values() {
            for child in children {
                all_children.insert(child.clone());
            }
        }

        // Identify root nodes (nodes that are not in the set of all children)
        for parent in parent_map.keys() {
            if !all_children.contains(parent) {
                root_nodes.push(vec![parent.to_string(), "test".to_string()]);
            }
        }
        self.tf_frames.items = vec![
            {
                let string1 = root_nodes[0][0].to_string();
                let text1 = Text::from(string1.clone());
                TreeItem::new_leaf(String::from(&string1),string1.clone())
            },
            {
                let string2 = root_nodes[1][0].to_string();
                let text2 = Text::from(string2.clone());
                TreeItem::new_leaf(String::from(&string2),string2.clone())
            },
        ];

        self.frames = root_nodes;

    }

    fn reset(&mut self) {}
    fn get_description(&self) -> Vec<String> {
        vec!["Topic manager can enable and disable displayed topics".to_string()]
    }

    fn handle_input(&mut self, input: &String) {
        // if self.selection_mode {
        //     match input.as_str() {
        //         input::UP => self.availible_topics.previous(),
        //         input::DOWN => self.availible_topics.next(),
        //         input::RIGHT => self.shift_active_element_right(),
        //         input::ROTATE_RIGHT => {
        //             self.selection_mode = false;
        //             self.selected_topics.state.select(Some(0));
        //             self.availible_topics.state.select(None);
        //         }
        //         input::CONFIRM => self.save(),
        //         _ => (),
        //     }
        // } else {
        //     match input.as_str() {
        //         input::UP => self.selected_topics.previous(),
        //         input::DOWN => self.selected_topics.next(),
        //         input::LEFT => self.shift_active_element_left(),
        //         input::ROTATE_LEFT => {
        //             self.selection_mode = true;
        //             self.availible_topics.state.select(Some(0));
        //             self.selected_topics.state.select(None);
        //         }
        //         input::CONFIRM => self.save(),
        //         _ => (),
        //     }
        // }
    }

    fn get_keymap(&self) -> Vec<[String; 2]> {
        vec![
            [
                input::UP.to_string(),
                "Selects the previous item in the active list".to_string(),
            ],
            [
                input::DOWN.to_string(),
                "Selects the next item in the active list".to_string(),
            ],
            [
                input::RIGHT.to_string(),
                "Shifts an element to the right if the supported topic list is active".to_string(),
            ],
            [
                input::LEFT.to_string(),
                "Shifts an element to the left if the active list is active".to_string(),
            ],
            [
                input::ROTATE_RIGHT.to_string(),
                "Changes the list where items are selected to the active topics list".to_string(),
            ],
            [
                input::ROTATE_LEFT.to_string(),
                "Changes the list where items are selected to the supported topics list"
                    .to_string(),
            ],
            [input::CONFIRM.to_string(), "Saves to config".to_string()],
        ]
    }

    fn get_name(&self) -> String {
        "Topic Manager".to_string()
    }
}

impl<B: Backend> Drawable<B> for TopicManager {
    fn draw(&self, f: &mut Frame<B>) {
        let title_text = vec![Spans::from(Span::styled(
            "TF tree viewer",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))];
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(20)
            .constraints(
                [
                    Constraint::Length(3), // Title + 2 borders
                    Constraint::Length(2),
                    Constraint::Min(1), // Table + header + space
                ]
                .as_ref(),
            )
            .split(f.size());
        let title = Paragraph::new(title_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        if !self.was_saved {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(100)].as_ref());

            // The `List` widget is then built with those items.
            let list = Tree::new(&self.tf_frames.items)
                .expect("all item identifiers are unique")
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .block(
                    Block::default()
                        .title("Current tf frames")
                        .borders(Borders::ALL),
                )
                .highlight_symbol(">> ");

            // Finally the widget is rendered using the associated state. `events.state` is
            // effectively the only thing that we will "remember" from this draw call.
            f.render_widget(title, areas[0]);
            let mut state = self.tf_frames.state;
            f.render_stateful_widget(
                list,
                areas[2],
                &mut state,
            );
        } else {
            let user_info = Paragraph::new(Spans::from(Span::raw(
                "Config has been saved, restart termviz to use it. \n Switch to any other mode to continue"
            )))
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
            f.render_widget(user_info, areas[1]);
        }
    }
}
