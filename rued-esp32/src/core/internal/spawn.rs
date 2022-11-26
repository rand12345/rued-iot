use core::fmt::Debug;

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_hal_0_2::adc;
use embedded_hal_0_2::digital::v2::{InputPin, OutputPin};

use embedded_svc::mqtt::client::asynch::{Client, Connection, Publish};
use embedded_svc::wifi::Wifi as WifiTrait;
use embedded_svc::ws::asynch::server::Acceptor;
use esp_idf_svc::wifi::{EspWifi, WifiEvent};

use gfx_xtra::draw_target::Flushable;

use edge_executor::*;

use channel_bridge::asynch::*;

use crate::core::internal::{mqtt::MqttCommand, screen};

use super::button::{self, PressedLevel};
use super::screen::Color;
use super::{battery, mqtt, wifi};

pub fn high_prio1<'a, const C: usize, M, D>(
    executor: &mut Executor<'a, C, M, Local>,
    tasks: &mut heapless::Vec<Task<()>, C>,
    button1_pin: impl InputPin<Error = impl Debug + 'a> + 'a,
    display: D,
    wifi: (EspWifi<'a>, impl Receiver<Data = WifiEvent> + 'a),
) -> Result<(), SpawnError>
where
    M: Monitor + Default,
    D: Flushable<Color = BinaryColor> + 'a,
    D::Error: Debug,
{
    executor
        .spawn_local_collect(
            super::button::button1_process(button1_pin, super::button::PressedLevel::Low),
            tasks,
        )?
        .spawn_local_collect(super::inspector::process(), tasks)?
        .spawn_local_collect(super::keepalive::process(), tasks)?
        .spawn_local_collect(screen::process(), tasks)?
        .spawn_local_collect(screen::run_draw(display), tasks)?
        .spawn_local_collect(super::wifi::process(wifi.0, wifi.1), tasks)?;

    Ok(())
}

pub fn high_prio2<'a, const C: usize, M>(
    executor: &mut Executor<'a, C, M, Local>,
    tasks: &mut heapless::Vec<Task<()>, C>,
    wifi: (EspWifi<'a>, impl Receiver<Data = WifiEvent> + 'a),
) -> Result<(), SpawnError>
where
    M: Monitor + Default,
{
    // FIXME - need to get wifi running.
    executor.spawn_local_collect(super::wifi::process(wifi.0, wifi.1), tasks)?;

    Ok(())
}

pub fn high_prio_original<'a, ADC, BP, const C: usize, M>(
    executor: &mut Executor<'a, C, M, Local>,
    tasks: &mut heapless::Vec<Task<()>, C>,
    battery_voltage: impl adc::OneShot<ADC, u16, BP> + 'a,
    battery_pin: BP,
    // used to indicate if powered by battery, only if this is enabled will Deep-sleep be enabled.
    power_pin: impl InputPin + 'a,
    roller: bool,
    button1_pin: impl InputPin<Error = impl Debug + 'a> + 'a,
    button2_pin: impl InputPin<Error = impl Debug + 'a> + 'a,
    button3_pin: impl InputPin<Error = impl Debug + 'a> + 'a,
) -> Result<(), SpawnError>
where
    M: Monitor + Default,
    ADC: 'a,
    BP: adc::Channel<ADC> + 'a,
{
    executor
        .spawn_local_collect(
            battery::process(battery_voltage, battery_pin, power_pin),
            tasks,
        )?
        .spawn_local_collect(
            super::button::button3_process(button3_pin, super::button::PressedLevel::Low),
            tasks,
        )?
        // .spawn_local_collect(emergency::process(), tasks)?
        .spawn_local_collect(super::keepalive::process(), tasks)?;

    if roller {
        executor.spawn_local_collect(
            super::button::button1_button2_roller_process(button1_pin, button2_pin),
            tasks,
        )?;
    } else {
        executor
            .spawn_local_collect(
                super::button::button1_process(button1_pin, super::button::PressedLevel::Low),
                tasks,
            )?
            .spawn_local_collect(
                super::button::button2_process(button2_pin, super::button::PressedLevel::Low),
                tasks,
            )?;
    }

    Ok(())
}

pub fn mid_prio<'a, const C: usize, M, D>(
    executor: &mut Executor<'a, C, M, Local>,
    tasks: &mut heapless::Vec<Task<()>, C>,
    display: D,
    // wm_flash: impl FnMut(WaterMeterState) + 'a,
) -> Result<(), SpawnError>
where
    M: Monitor + Default,
    D: Flushable<Color = BinaryColor> + 'a,
    D::Error: Debug,
{
    executor
        .spawn_local_collect(screen::process(), tasks)?
        .spawn_local_collect(screen::run_draw(display), tasks)?;

    // .spawn_local_collect(wm_stats::process(), tasks)?
    // .spawn_local_collect(wm::flash(wm_flash), tasks)?;

    Ok(())
}

pub fn wifi<'a, const C: usize, M, D>(
    executor: &mut Executor<'a, C, M, Local>,
    tasks: &mut heapless::Vec<Task<()>, C>,
    wifi: EspWifi<'a>,
    wifi_notif: impl Receiver<Data = D> + 'a,
) -> Result<(), SpawnError>
where
    M: Monitor + Default,
    D: 'a,
    WifiEvent: From<D>,
{
    executor.spawn_local_collect(super::wifi::process(wifi, wifi_notif), tasks)?;

    Ok(())
}

pub fn mqtt_send<'a, const L: usize, const C: usize, M>(
    executor: &mut Executor<'a, C, M, Local>,
    tasks: &mut heapless::Vec<Task<()>, C>,
    mqtt_topic_prefix: &'a str,
    mqtt_client: impl Client + Publish + 'a,
) -> Result<(), SpawnError>
where
    M: Monitor + Default,
{
    executor.spawn_local_collect(mqtt::send::<L>(mqtt_topic_prefix, mqtt_client), tasks)?;

    Ok(())
}

pub fn mqtt_receive<'a, const C: usize, M>(
    executor: &mut Executor<'a, C, M, Local>,
    tasks: &mut heapless::Vec<Task<()>, C>,
    mqtt_conn: impl Connection<Message = Option<MqttCommand>> + 'a,
) -> Result<(), SpawnError>
where
    M: Monitor + Default,
{
    executor.spawn_local_collect(mqtt::receive(mqtt_conn), tasks)?;

    Ok(())
}

pub fn run<const C: usize, M>(
    executor: &mut Executor<C, M, Local>,
    tasks: heapless::Vec<Task<()>, C>,
) where
    M: Monitor + Wait + Default,
{
    // FIXME: Simulate a button press
    // let condition = move || false;

    let condition = move || !super::quit::QUIT.triggered();
    executor.run_tasks(condition, tasks);
}

pub fn start<const C: usize, M, F>(
    executor: &'static mut Executor<C, M, Local>,
    tasks: heapless::Vec<Task<()>, C>,
    finished: F,
) where
    M: Monitor + Start + Default,
    F: FnOnce() + 'static,
{
    executor.start(move || !super::quit::QUIT.triggered(), {
        let executor = &*executor;

        move || {
            executor.drop_tasks(tasks);

            finished();
        }
    });
}
