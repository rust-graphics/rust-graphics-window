#[cfg(feature = "verbose_log")]
use log::log_i;
use log::{result_f, unwrap_f};
use std::{
    collections::{BTreeMap, BTreeSet, LinkedList},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{channel, Sender},
        Arc, Mutex, RwLock, Weak,
    },
    thread::{spawn, JoinHandle},
    time::{Duration, Instant},
};

pub type FingerIndexType = i64;

#[cfg_attr(feature = "debug_derive", derive(Debug))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum Mouse {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Offic,
    Unknown(u32),
}

#[cfg_attr(feature = "debug_derive", derive(Debug))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum Keyboard {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Escape(u8),
    Function(u8),
    PrintScreen,
    ScrollLock,
    PauseBreak,
    BackQuote,
    Number { number: i32, padd: bool },
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    NumLock,
    Slash(u8),
    Star,
    Plus(u8),
    Minus(u8),
    Enter(u8),
    Dot(u8),
    Tab,
    BracketLeft,
    BracketRight,
    CapseLock(u8),
    SemiColon,
    Quote,
    BackSlash(u8),
    Shift(u8),
    Comma(u8),
    Control(u8),
    Alt(u8),
    Space(u8),
    Command(u8),
    Super(u8),
    Properties(u8),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Equal,
    Menu(u8),
    Unknown(u32),
}

#[cfg_attr(feature = "debug_derive", derive(Debug))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum Button {
    Mouse(Mouse),
    Keyboard(Keyboard),
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub struct WindowSizeChange {
    pub current: WindowAspects,
    pub previous: WindowAspects,
    pub delta: WindowAspects,
}

#[derive(Clone)]
#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub enum Window {
    SizeChange(WindowSizeChange),
}

#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub enum Move {
    Mouse {
        previous: (i64, i64),
        current: (i64, i64),
        delta: (i64, i64),
    },
    Touch {
        index: FingerIndexType,
        previous: (i64, i64),
        current: (i64, i64),
        delta: (i64, i64),
    },
}

#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub enum ButtonAction {
    Press,
    Release,
}

#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub enum TouchAction {
    Press,
    HardPress,
    Release,
}

#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub enum TouchGesture {
    Tap, // todo
    Drag {
        index: FingerIndexType,
        start: (i64, i64),
        previous: (i64, i64),
        current: (i64, i64),
        delta: (i64, i64),
    },
    Scale {
        first: (FingerIndexType, (i64, i64)),
        second: (FingerIndexType, (i64, i64)),
        start: i64,
        previous: i64,
        current: i64,
        delta: i64,
    },
}

#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub enum GestureState {
    Started,
    InMiddle,
    Ended,
    Canceled,
}

#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub enum Touch {
    Gesture {
        start_time: Instant,
        duration: Duration,
        state: GestureState,
        gest: TouchGesture,
    },
    Raw {
        index: FingerIndexType,
        action: TouchAction,
        point: (i64, i64),
    },
}

#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub enum Data {
    Move(Move),
    Button {
        button: Button,
        action: ButtonAction,
    },
    Touch(Touch),
    Window(Window),
    Quit,
    Terminate,
}

#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub struct Event {
    id: u64,
    time: Instant,
    data: Data,
}

pub static NEXT_ID: AtomicU64 = AtomicU64::new(1);

impl Event {
    pub fn new(data: Data) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            time: Instant::now(),
            data,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_time(&self) -> &Instant {
        &self.time
    }

    pub fn get_data(&self) -> &Data {
        &self.data
    }
}

pub trait Listener: Send + Sync {
    fn on_event(&mut self, event: &Event) -> bool;
}

#[derive(Default, Clone, Copy)]
#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub struct WindowAspects {
    width: i64,
    height: i64,
    ratio: f64,
}

#[derive(Default)]
struct WindowState {
    aspects: WindowAspects,
    changing_aspects: Option<WindowAspects>,
}

#[derive(Default)]
struct EngineState {
    window: WindowState,
    mouse_position_x: i64,
    mouse_position_y: i64,
    pressed_buttons: BTreeSet<Button>,
}

pub struct Engine {
    listeners: Arc<Mutex<BTreeMap<i64, LinkedList<Weak<RwLock<dyn Listener>>>>>>,
    processor: Option<JoinHandle<()>>,
    sender: Sender<Event>,
    state: Arc<Mutex<EngineState>>,
}

impl Engine {
    pub(crate) fn new() -> Self {
        let listeners: Arc<Mutex<BTreeMap<i64, LinkedList<Weak<RwLock<dyn Listener>>>>>> =
            Arc::new(Mutex::new(BTreeMap::new()));
        let (sender, receiver) = channel();
        let ls = listeners.clone();
        let processor = Some(spawn(move || {
            let mut pending_window_resize: Option<WindowSizeChange> = None;
            let wait_dur = Duration::from_millis(100);
            'engine_loop: loop {
                let e: Option<Event> = match receiver.recv_timeout(wait_dur) {
                    Ok(e) => Some(e),
                    Err(_) => None,
                };
                let e: Event = if let Some(e) = e {
                    match e.get_data() {
                        &Data::Terminate => return,
                        &Data::Window(Window::SizeChange(ref e)) => {
                            if pending_window_resize.is_some() {
                                let p = unwrap_f!(pending_window_resize.as_mut());
                                p.current = e.current.clone();
                                p.delta = WindowAspects {
                                    width: p.current.width - p.previous.width,
                                    height: p.current.height - p.previous.height,
                                    ratio: p.current.ratio - p.previous.ratio,
                                };
                            } else {
                                pending_window_resize = Some(e.clone());
                            }
                            continue 'engine_loop;
                        }
                        _ => (),
                    }
                    e
                } else if pending_window_resize.is_some() {
                    let e = Event::new(Data::Window(Window::SizeChange(unwrap_f!(
                        pending_window_resize.clone()
                    ))));
                    pending_window_resize = None;
                    e
                } else {
                    continue 'engine_loop;
                };
                let listeners = result_f!(ls.lock());
                'listeners_loop: for (_, ls) in &*listeners {
                    for l in ls {
                        if let Some(l) = l.upgrade() {
                            if result_f!(l.write()).on_event(&e) {
                                break 'listeners_loop;
                            }
                        }
                    }
                }
            }
        }));
        let state = Arc::new(Mutex::new(EngineState::default()));
        Self {
            listeners,
            processor,
            sender,
            state,
        }
    }

    pub(crate) fn broadcast(&self, e: Event) {
        result_f!(self.sender.send(e));
    }

    pub fn add(&self, priority: i64, l: Weak<RwLock<dyn Listener>>) {
        (*result_f!(self.listeners.lock()))
            .entry(priority)
            .or_insert(LinkedList::new())
            .push_back(l);
    }

    pub fn clean(&self) {
        let mut listeners = result_f!(self.listeners.lock());
        for (_, listeners) in &mut *listeners {
            listeners.drain_filter(|x| x.strong_count() <= 0);
        }
    }

    pub(crate) fn init_window_aspects(&self, width: i64, height: i64) {
        let mut state = result_f!(self.state.lock());
        state.window.aspects.width = width;
        state.window.aspects.height = height;
        state.window.aspects.ratio = width as f64 / height as f64;
    }

    pub(crate) fn init_mouse_position(&self, p: (i64, i64)) {
        let mut state = result_f!(self.state.lock());
        state.mouse_position_x = p.0;
        state.mouse_position_y = p.1;
    }

    pub(crate) fn set_mouse_position(&self, cur: (i64, i64)) {
        self.broadcast(Event::new({
            let mut state = result_f!(self.state.lock());
            let d = Data::Move(Move::Mouse {
                previous: (state.mouse_position_x, state.mouse_position_y),
                current: cur,
                delta: (
                    cur.0 - state.mouse_position_x,
                    cur.1 - state.mouse_position_y,
                ),
            });
            state.mouse_position_x = cur.0;
            state.mouse_position_y = cur.1;
            d
        }));
    }

    pub(crate) fn finger_down(&self, x: i64, y: i64, finget_index: FingerIndexType) {}

    pub(crate) fn finger_up(&self, x: i64, y: i64, finget_index: FingerIndexType) {}

    pub(crate) fn finger_move(&self, x: i64, y: i64, finget_index: FingerIndexType) {}

    pub(crate) fn button_pressed(&self, b: Button) {
        self.broadcast(Event::new({
            let mut state = result_f!(self.state.lock());
            state.pressed_buttons.insert(b.clone());
            Data::Button {
                button: b,
                action: ButtonAction::Press,
            }
        }));
    }

    pub(crate) fn button_released(&self, b: Button) {
        self.broadcast(Event::new({
            let mut state = result_f!(self.state.lock());
            state.pressed_buttons.remove(&b);
            Data::Button {
                button: b,
                action: ButtonAction::Release,
            }
        }));
    }

    pub(crate) fn window_size_changed(&self, width: i64, height: i64) {
        if width <= 0 || height <= 0 {
            return;
        }
        self.broadcast(Event::new(Data::Window({
            let mut state = result_f!(self.state.lock());
            if width == state.window.aspects.width && height == state.window.aspects.height {
                return;
            }
            let ratio = width as f64 / height as f64;
            state.window.changing_aspects = Some(WindowAspects {
                width,
                height,
                ratio,
            });
            let d = Window::SizeChange(WindowSizeChange {
                current: WindowAspects {
                    width,
                    height,
                    ratio,
                },
                previous: WindowAspects {
                    width: state.window.aspects.width,
                    height: state.window.aspects.height,
                    ratio: state.window.aspects.ratio,
                },
                delta: WindowAspects {
                    width: width - state.window.aspects.width,
                    height: height - state.window.aspects.height,
                    ratio: ratio - state.window.aspects.ratio,
                },
            });
            state.window.aspects.width = width;
            state.window.aspects.height = height;
            state.window.aspects.ratio = ratio;
            state.window.changing_aspects = None;
            d
        })));
    }

    pub(crate) fn quit(&self) {
        self.broadcast(Event::new(Data::Quit));
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        result_f!(self.sender.send(Event::new(Data::Terminate)));
        if let Some(processor) = self.processor.take() {
            result_f!(processor.join());
        }
        #[cfg(feature = "verbose_log")]
        log_i!("Rust-Graphics's Window library's Event Engine droped.");
    }
}
