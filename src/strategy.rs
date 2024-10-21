use std::collections::HashSet;
use std::ffi::CStr;
use std::mem::transmute;
use std::sync::OnceLock;
use std::{ffi::CString, os::raw::c_char};

use chrono::{self, Datelike};
use serde::{Deserialize, Serialize};
use vnrs::vnrs::trader::constant::{Direction, Interval, Offset};
use vnrs::vnrs::trader::object::{BarData, OrderData, TickData, TradeData};
use vnrs::vnrs::trader::utility::{ArrayManager, BarGenerator};
use vnrs::vnrs_ctastrategy::backtesting::BacktestingEngine;
use vnrs::vnrs_ctastrategy::base::StopOrder;

#[no_mangle]
pub extern "C" fn abi_version() -> u64 {
    1
}

#[no_mangle]
pub extern "C" fn abi_author() -> *const c_char {
    let c_string = AUTHOR.get_or_init(|| CString::new(author()).unwrap());
    c_string.as_ptr()
}

#[no_mangle]
pub extern "C" fn abi_parameters() -> *const c_char {
    let c_string = PARAMETERS.get_or_init(|| CString::new(parameters()).unwrap());
    c_string.as_ptr()
}

#[no_mangle]
pub extern "C" fn abi_variables() -> *const c_char {
    let c_string = VARIABLES
        .get_or_init(|| CString::new(format!("inited,trading,pos,{}", variables())).unwrap());
    c_string.as_ptr()
}

#[no_mangle]
#[allow(private_interfaces, improper_ctypes_definitions)]
pub extern "C" fn abi_new(
    cta_engine: usize,
    strategy_name: *mut c_char,
    vt_symbol: *mut c_char,
    setting: *mut c_char,
) -> *mut CtaTemplate {
    unsafe {
        Box::into_raw(Box::new(CtaTemplate::new(
            cta_engine,
            CStr::from_ptr(strategy_name)
                .to_owned()
                .into_string()
                .unwrap(),
            CStr::from_ptr(vt_symbol).to_owned().into_string().unwrap(),
            CStr::from_ptr(setting).to_owned().into_string().unwrap(),
        )))
    }
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_drop(this: *mut CtaTemplate) {
    drop(unsafe { Box::from_raw(this) });
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_get_inited_mut(this: *mut CtaTemplate) -> *mut bool {
    let this = unsafe { &mut *this };
    &mut this.inited
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_get_trading_mut(this: *mut CtaTemplate) -> *mut bool {
    let this = unsafe { &mut *this };
    &mut this.trading
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_get_pos_mut(this: *mut CtaTemplate) -> *mut f64 {
    let this = unsafe { &mut *this };
    &mut this.pos
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_on_init(this: *mut CtaTemplate, cta_engine_ptr: usize) {
    let this = unsafe { &mut *this };
    this.cta_engine.this = cta_engine_ptr;
    this.on_init();
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_on_start(this: *mut CtaTemplate) {
    let this = unsafe { &mut *this };
    this.on_start();
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_on_stop(this: *mut CtaTemplate) {
    let this = unsafe { &mut *this };
    this.on_stop();
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_on_tick(this: *mut CtaTemplate, tick: *const TickData) {
    unsafe {
        let this = &mut *this;
        this.on_tick(&*tick);
    }
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_on_bar(this: *mut CtaTemplate, bar: *const BarData) {
    unsafe {
        let this = &mut *this;
        this.on_bar(&*bar);
    }
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_on_order(this: *mut CtaTemplate, order: *const OrderData) {
    unsafe {
        let this = &mut *this;
        this.on_order(&*order);
    }
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_on_trade(this: *mut CtaTemplate, trade: *const TradeData) {
    unsafe {
        let this = &mut *this;
        this.on_trade(&*trade);
    }
}

#[no_mangle]
#[allow(private_interfaces)]
pub extern "C" fn abi_on_stop_order(this: *mut CtaTemplate, stop_order: *const StopOrder) {
    unsafe {
        let this = &mut *this;
        this.on_stop_order(&*stop_order);
    }
}

static AUTHOR: OnceLock<CString> = OnceLock::new();
static PARAMETERS: OnceLock<CString> = OnceLock::new();
static VARIABLES: OnceLock<CString> = OnceLock::new();

const V_TABLE_LEN: usize = 5;

#[derive(Debug, Default)]
pub struct CtaEngineExtern {
    pub this: usize,
    v_table: [usize; V_TABLE_LEN],
}
impl CtaEngineExtern {
    pub fn new(u: usize) -> Self {
        let mut v_table: [usize; V_TABLE_LEN] = [0; V_TABLE_LEN];
        unsafe {
            let p = u as *mut usize;
            std::ptr::copy_nonoverlapping(p, v_table.as_mut_ptr(), V_TABLE_LEN);
            CtaEngineExtern {
                this: 0,
                v_table: v_table,
            }
        }
    }

    pub fn load_bar(
        &self,
        vt_symbol: String,
        days: i64,
        interval: Interval,
        use_database: bool,
    ) -> Vec<BarData> {
        if self.this == 0 {
            panic!("cta_engine ptr is null!");
        }
        let cloned_vec;
        unsafe {
            let abi_load_bar = transmute::<
                usize,
                unsafe extern "C" fn(
                    usize,
                    *const c_char,
                    i64,
                    Interval,
                    bool,
                ) -> *mut Vec<BarData>,
            >(self.v_table[0]);
            let vec_bar_data = abi_load_bar(
                self.this,
                CString::new(vt_symbol).unwrap().as_ptr(),
                days,
                interval,
                use_database,
            );
            cloned_vec = (&*vec_bar_data).clone();
            let abi_drop_vec_bar_data =
                transmute::<usize, unsafe extern "C" fn(*mut Vec<BarData>)>(self.v_table[1]);
            abi_drop_vec_bar_data(vec_bar_data);
        }
        cloned_vec
    }

    pub fn send_order(
        &self,
        strategy: *const CtaTemplate,
        direction: Direction,
        offset: Offset,
        price: f64,
        volume: f64,
        stop: bool,
        lock: bool,
        net: bool,
    ) -> Vec<String> {
        if self.this == 0 {
            panic!("cta_engine ptr is null!");
        }
        let cloned_vec;
        unsafe {
            let abi_send_order = transmute::<
                usize,
                unsafe extern "C" fn(
                    usize,
                    *const CtaTemplate,
                    Direction,
                    Offset,
                    f64,
                    f64,
                    bool,
                    bool,
                    bool,
                ) -> *mut Vec<String>,
            >(self.v_table[2]);
            let vec_string = abi_send_order(
                self.this, strategy, direction, offset, price, volume, stop, lock, net,
            );
            cloned_vec = (&*vec_string).clone();
            let abi_drop_vec_string =
                transmute::<usize, unsafe extern "C" fn(*mut Vec<String>)>(self.v_table[3]);
            abi_drop_vec_string(vec_string);
        }
        cloned_vec
    }

    pub fn cancel_all(&self, strategy: *const CtaTemplate) {
        unsafe {
            let abi_cancel_all = transmute::<usize, unsafe extern "C" fn(usize, *const CtaTemplate)>(
                self.v_table[4],
            );
            abi_cancel_all(self.this, strategy);
        }
    }
}

const fn author() -> &'static str {
    "wuliehan"
}

const fn parameters() -> &'static str {
    "fast_window:10,slow_window:20"
}

const fn variables() -> &'static str {
    "fast_ma0,fast_ma1,slow_ma0,slow_ma1"
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CtaTemplate {
    #[serde(default, skip)]
    pub cta_engine: CtaEngineExtern,
    strategy_name: String,
    vt_symbol: String,

    inited: bool,
    trading: bool,
    pos: f64,

    fast_window: i64,
    slow_window: i64,

    #[serde(default)]
    fast_ma0: f64,
    #[serde(default)]
    fast_ma1: f64,

    #[serde(default)]
    slow_ma0: f64,
    #[serde(default)]
    slow_ma1: f64,

    #[serde(default, skip)]
    bg: Option<BarGenerator>,
    #[serde(default, skip)]
    am: Option<ArrayManager>,
}

impl CtaTemplate {
    pub fn new(
        cta_engine: usize,
        strategy_name: String,
        vt_symbol: String,
        setting: String,
    ) -> Self {
        eprintln!("{}", cta_engine);
        let mut this = Self::super_init(cta_engine, strategy_name, vt_symbol, setting);
        this.bg = Some(BarGenerator::new(unsafe {
            transmute::<fn(&mut CtaTemplate, &BarData), fn(usize, &BarData)>(Self::on_bar)
        }));
        this.am = Some(ArrayManager::new(100));
        this
    }

    fn on_init(&mut self) {
        self.write_log("策略初始化".to_string());
        // self.load_bar(10, Interval::DAILY, CtaTemplate::on_bar, false);
    }

    fn on_start(&mut self) {
        self.write_log("策略启动".to_string());
        self.put_event();
    }

    fn on_stop(&mut self) {
        self.write_log("策略停止".to_string());

        self.put_event();
    }

    fn on_tick(&mut self, tick: &TickData) {
        // self.bg.unwrap().update_tick(tick);
    }

    fn on_bar(&mut self, bar: &BarData) {
        self.cancel_all();

        let am = self.am.as_mut().unwrap();
        am.update_bar(&bar);
        if !am.inited {
            return;
        }

        let fast_ma = am.sma_array(self.fast_window);
        self.fast_ma0 = fast_ma[fast_ma.len() - 1];
        self.fast_ma1 = fast_ma[fast_ma.len() - 2];

        let slow_ma = am.sma_array(self.slow_window);
        self.slow_ma0 = slow_ma[slow_ma.len() - 1];
        self.slow_ma1 = slow_ma[slow_ma.len() - 2];

        let cross_over = self.fast_ma0 > self.slow_ma0 && self.fast_ma1 < self.slow_ma1;
        let cross_below = self.fast_ma0 < self.slow_ma0 && self.fast_ma1 > self.slow_ma1;

        if cross_over {
            if self.pos == 0.0 {
                self.buy(bar.close_price, 1.0, false, false, false);
            } else if self.pos < 0.0 {
                self.cover(bar.close_price, 1.0, false, false, false);
                self.buy(bar.close_price, 1.0, false, false, false);
            }
        } else if cross_below {
            if self.pos == 0.0 {
                self.short(bar.close_price, 1.0, false, false, false);
            } else if self.pos > 0.0 {
                self.sell(bar.close_price, 1.0, false, false, false);
                self.short(bar.close_price, 1.0, false, false, false);
            }
        }

        self.put_event();
    }

    fn on_order(&mut self, order: &OrderData) {}

    fn on_trade(&mut self, trade: &TradeData) {
        self.put_event();
    }

    fn on_stop_order(&mut self, stop_order: &StopOrder) {}
}

impl CtaTemplate {
    fn super_init(
        cta_engine: usize,
        strategy_name: String,
        vt_symbol: String,
        setting: String,
    ) -> Self {
        let mut assigned_set = HashSet::new();
        let mut parameters_json_string = "".to_string();
        if !setting.is_empty() {
            for kv in setting.split(",") {
                let pair: Vec<&str> = kv.split(":").collect();
                if assigned_set.contains(pair[0]) {
                    continue;
                }
                assigned_set.insert(pair[0]);
                parameters_json_string.push_str(&format!(r#""{}":{},"#, pair[0], pair[1]));
            }
        }
        for kv in parameters().split(",") {
            let pair: Vec<&str> = kv.split(":").collect();
            if assigned_set.contains(pair[0]) {
                continue;
            }
            assigned_set.insert(pair[0]);
            parameters_json_string.push_str(&format!(r#""{}":{},"#, pair[0], pair[1]));
        }
        let mut json = format!(
            r#"{{"cta_engine":{},"strategy_name":"{}","vt_symbol":"{}","inited":false,"trading":false,"pos":0,{}"#,
            cta_engine, strategy_name, vt_symbol, parameters_json_string
        );
        json.pop();
        json.push('}');
        println!("{}", json);
        let mut ret: CtaTemplate = serde_json::from_str(&json).unwrap();
        ret.cta_engine = CtaEngineExtern::new(cta_engine);
        ret
    }
    fn buy(&self, price: f64, volume: f64, stop: bool, lock: bool, net: bool) -> Vec<String> {
        self.send_order(
            Direction::LONG,
            Offset::OPEN,
            price,
            volume,
            stop,
            lock,
            net,
        )
    }

    fn sell(&self, price: f64, volume: f64, stop: bool, lock: bool, net: bool) -> Vec<String> {
        return self.send_order(
            Direction::SHORT,
            Offset::CLOSE,
            price,
            volume,
            stop,
            lock,
            net,
        );
    }

    fn short(&self, price: f64, volume: f64, stop: bool, lock: bool, net: bool) -> Vec<String> {
        return self.send_order(
            Direction::SHORT,
            Offset::OPEN,
            price,
            volume,
            stop,
            lock,
            net,
        );
    }

    fn cover(&self, price: f64, volume: f64, stop: bool, lock: bool, net: bool) -> Vec<String> {
        return self.send_order(
            Direction::LONG,
            Offset::CLOSE,
            price,
            volume,
            stop,
            lock,
            net,
        );
    }

    fn send_order(
        &self,
        direction: Direction,
        offset: Offset,
        price: f64,
        volume: f64,
        stop: bool,
        lock: bool,
        net: bool,
    ) -> Vec<String> {
        if self.trading {
            let vt_orderids: Vec<String> = self
                .cta_engine
                .send_order(self, direction, offset, price, volume, stop, lock, net);
            vt_orderids
        } else {
            vec![]
        }
    }

    fn cancel_all(&self) {
        self.cta_engine.cancel_all(self)
    }

    fn write_log(&self, msg: String) {
        // unsafe {
        //     let ptr_table = self.cta_engine as *const usize;
        //     let fun_print_log =
        //         transmute::<usize, unsafe extern "C" fn(*const c_char)>(*ptr_table.offset(0));
        //     fun_print_log(CString::new(msg).unwrap().as_ptr());
        // }
    }

    ///Load historical bar data for initializing strategy.
    fn load_bar(
        &mut self,
        days: i64,
        interval: Interval,
        callback: fn(&mut CtaTemplate, bar: &BarData),
        use_database: bool,
    ) {
        let bars: Vec<BarData> =
            self.cta_engine
                .load_bar(self.vt_symbol.to_string(), days, interval, use_database);

        for bar in bars {
            callback(self, &bar);
        }
    }

    fn put_event(&self) {}
}

impl Drop for CtaTemplate {
    fn drop(&mut self) {
        println!("dropped:{:p} ", self);
    }
}
