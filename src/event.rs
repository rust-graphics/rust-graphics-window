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
    Focus,
    Unfocus,
}

#[cfg_attr(feature = "debug_derive", derive(Debug))]
pub enum Move {
    Mouse {
        previous: (i64, i64),
        current: (i64, i64),
        delta: (i64, i64),
        normalized_previous: (f64, f64),
        normalized_current: (f64, f64),
        normalized_delta: (f64, f64),
    },
    Touch {
        index: FingerIndexType,
        previous: (i64, i64),
        current: (i64, i64),
        delta: (i64, i64),
        normalized_previous: (f64, f64),
        normalized_current: (f64, f64),
        normalized_delta: (f64, f64),
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
        normalized_start: (f64, f64),
        normalized_previous: (f64, f64),
        normalized_current: (f64, f64),
        normalized_delta: (f64, f64),
    },
    Scale {
        first: (FingerIndexType, TouchState),
        second: (FingerIndexType, TouchState),
        start: i64,
        previous: i64,
        current: i64,
        delta: i64,
        normalized_start: i64,
        normalized_previous: i64,
        normalized_current: i64,
        normalized_delta: i64,
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
        data: TouchGesture,
    },
    Raw {
        index: FingerIndexType,
        action: TouchAction,
        point: (i64, i64),
        normalized_point: (f64, f64),
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
    smallest: i64,
    ratio: f64,
    normalized_width: f64,
    normalized_height: f64,
}

#[derive(Default)]
struct WindowState {
    aspects: WindowAspects,
}

impl WindowState {
    fn normalize(&self, x: i64, y: i64) -> (f64, f64) {
        (self.normalize_width(x), self.normalize_height(y))
    }

    fn normalize_width(&self, x: i64) -> f64 {
        (x - self.aspects.width) as f64 / self.aspects.smallest as f64
    }

    fn normalize_height(&self, x: i64) -> f64 {
        (x - self.aspects.height) as f64 / self.aspects.smallest as f64
    }
}

#[cfg_attr(feature = "verbose_log", derive(Debug))]
#[derive(Default, Clone)]
pub struct TouchState {
    position: (i64, i64),
    normalized_position: (f64, f64),
    hard_pressed: bool,
}

#[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
#[derive(Default)]
struct PointerState {
    position: (i64, i64),
    normalized_position: (f64, f64),
}

#[derive(Default)]
struct EngineState {
    window: WindowState,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    fingers: BTreeMap<FingerIndexType, TouchState>,
    #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
    mouse: PointerState,
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
                                    smallest: p.current.smallest - p.previous.smallest,
                                    normalized_width: p.current.normalized_width
                                        - p.previous.normalized_width,
                                    normalized_height: p.current.normalized_height
                                        - p.previous.normalized_height,
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
        let smallest = if width > height { height } else { width };
        let normalized_width = width as f64 / smallest as f64;
        let normalized_height = height as f64 / smallest as f64;
        let mut state = result_f!(self.state.lock());
        state.window.aspects.width = width;
        state.window.aspects.height = height;
        state.window.aspects.ratio = width as f64 / height as f64;
        state.window.aspects.smallest = smallest;
        state.window.aspects.normalized_width = normalized_width;
        state.window.aspects.normalized_height = normalized_height;
    }

    #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
    pub(crate) fn init_mouse_position(&self, p: (i64, i64)) {
        let mut state = result_f!(self.state.lock());
        state.mouse.position = p;
        state.mouse.normalized_position = state.window.normalize(p.0, p.1);
    }

    #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
    pub(crate) fn set_mouse_position(&self, cur: (i64, i64)) {
        self.broadcast(Event::new({
            let mut state = result_f!(self.state.lock());
            let nrm_cur = state.window.normalize(cur.0, cur.1);
            let d = Data::Move(Move::Mouse {
                previous: state.mouse.position,
                normalized_previous: state.mouse.normalized_position,
                current: cur,
                normalized_current: nrm_cur,
                delta: (
                    cur.0 - state.mouse.position.0,
                    cur.1 - state.mouse.position.1,
                ),
                normalized_delta: (
                    nrm_cur.0 - state.mouse.normalized_position.0,
                    nrm_cur.1 - state.mouse.normalized_position.1,
                ),
            });
            state.mouse.position = cur;
            state.mouse.normalized_position = nrm_cur;
            d
        }));
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(crate) fn finger_down(&self, x: i64, y: i64, index: FingerIndexType) {
        let nrm = {
            let mut s = result_f!(self.state.lock());
            let nrm = s.window.normalize(x, y);
            s.fingers.insert(
                index,
                TouchState {
                    position: (x, y),
                    normalized_position: nrm,
                    hard_pressed: false,
                },
            );
            nrm
        };
        self.broadcast(Event::new(Data::Touch(Touch::Raw {
            index,
            action: TouchAction::Press,
            point: (x, y),
            normalized_point: nrm,
        })));
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(crate) fn finger_up(&self, x: i64, y: i64, index: FingerIndexType) {
        let nrm = {
            let mut s = result_f!(self.state.lock());
            s.fingers.remove(&index);
            s.window.normalize(x, y)
        };
        self.broadcast(Event::new(Data::Touch(Touch::Raw {
            index,
            action: TouchAction::Release,
            point: (x, y),
            normalized_point: nrm,
        })));
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(crate) fn finger_move(&self, x: i64, y: i64, index: FingerIndexType) {
        let m = {
            let mut s = result_f!(self.state.lock());
            let nrm = s.window.normalize(x, y);
            let finger = if let Some(finger) = s.fingers.get(&index) {
                finger.clone()
            } else {
                s.fingers.insert(
                    index,
                    TouchState {
                        position: (x, y),
                        normalized_position: nrm,
                        hard_pressed: false,
                    },
                );
                return;
            };
            let delta = (x - finger.position.0, y - finger.position.1);
            let normalized_delta = (
                nrm.0 - finger.normalized_position.0,
                nrm.1 - finger.normalized_position.1,
            );
            let m = Move::Touch {
                index,
                previous: finger.position,
                current: (x, y),
                delta,
                normalized_previous: finger.normalized_position,
                normalized_current: nrm,
                normalized_delta,
            };
            s.fingers.insert(
                index,
                TouchState {
                    position: (x, y),
                    normalized_position: nrm,
                    hard_pressed: false,
                },
            );
            m
        };
        self.broadcast(Event::new(Data::Move(m)));
    }

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
            let smallest = if width > height { height } else { width };
            let normalized_width = width as f64 / smallest as f64;
            let normalized_height = height as f64 / smallest as f64;
            let d = Window::SizeChange(WindowSizeChange {
                current: WindowAspects {
                    width,
                    height,
                    ratio,
                    smallest,
                    normalized_width,
                    normalized_height,
                },
                previous: WindowAspects {
                    width: state.window.aspects.width,
                    height: state.window.aspects.height,
                    ratio: state.window.aspects.ratio,
                    smallest: state.window.aspects.smallest,
                    normalized_width: state.window.aspects.normalized_width,
                    normalized_height: state.window.aspects.normalized_height,
                },
                delta: WindowAspects {
                    width: width - state.window.aspects.width,
                    height: height - state.window.aspects.height,
                    ratio: ratio - state.window.aspects.ratio,
                    smallest: smallest - state.window.aspects.smallest,
                    normalized_width: normalized_width - state.window.aspects.normalized_width,
                    normalized_height: normalized_height - state.window.aspects.normalized_height,
                },
            });
            state.window.aspects.width = width;
            state.window.aspects.height = height;
            state.window.aspects.ratio = ratio;
            state.window.aspects.smallest = smallest;
            state.window.aspects.normalized_width = normalized_width;
            state.window.aspects.normalized_height = normalized_height;
            d
        })));
    }

    pub(crate) fn quit(&self) {
        self.broadcast(Event::new(Data::Quit));
    }

    pub(crate) fn window_focus(&self) {
        self.broadcast(Event::new(Data::Window(Window::Focus)));
    }

    pub(crate) fn window_unfocus(&self) {
        self.broadcast(Event::new(Data::Window(Window::Unfocus)));
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
