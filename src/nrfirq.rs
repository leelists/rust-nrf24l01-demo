use std::ptr;

use esp_idf_sys::{self as _, c_types::c_void, esp, gpio_config, gpio_config_t, gpio_install_isr_service, gpio_int_type_t_GPIO_INTR_NEGEDGE, gpio_isr_handler_add, gpio_mode_t_GPIO_MODE_INPUT, xQueueGenericCreate, xQueueGiveFromISR, xQueueReceive, QueueHandle_t, EspError};

// This `static mut` holds the queue handle we are going to get from `xQueueGenericCreate`.
// This is unsafe, but we are careful not to enable our GPIO interrupt handler until after this value has been initialised, and then never modify it again
static mut EVENT_QUEUE: Option<QueueHandle_t> = None;

// #[link_section = ".iram0.text"]
unsafe extern "C" fn nrf24_irq(_: *mut c_void) {
    xQueueGiveFromISR(EVENT_QUEUE.unwrap(), ptr::null_mut());
}

pub fn isr_init() -> Result<(), EspError> {
    // Queue configurations
    const QUEUE_TYPE_BASE: u8 = 0;
    const ITEM_SIZE: u32 = 0; // we're not posting any actual data, just notifying
    const QUEUE_SIZE: u32 = 1;
    // irq gpio
    const GPIO_NUM: i32 = 33;

    // Configures the gp
    let io_conf = gpio_config_t {
        pin_bit_mask: 1 << GPIO_NUM,
        mode: gpio_mode_t_GPIO_MODE_INPUT,
        pull_up_en: false.into(),
        pull_down_en: false.into(),
        intr_type: gpio_int_type_t_GPIO_INTR_NEGEDGE,
    };

    unsafe {
        // Writes the gpio configuration to the registers
        esp!(gpio_config(&io_conf))?;

        // Installs the generic GPIO interrupt handler
        esp!(gpio_install_isr_service(0 as i32))?;

        // Instantiates the event queue
        EVENT_QUEUE = Some(xQueueGenericCreate(QUEUE_SIZE, ITEM_SIZE, QUEUE_TYPE_BASE));

        // Registers our function with the generic GPIO interrupt handler we installed earlier.
        esp!(gpio_isr_handler_add(
            GPIO_NUM,
            Some(nrf24_irq),
            std::ptr::null_mut()
        ))?;
    }
    Ok(())
}

/// wait for IRQ interrupt returns true if interrupt ticked
pub fn isr_wait() -> bool {
    // maximum delay
    const QUEUE_WAIT_TICKS: u32 = 1000;

    unsafe {
        // Reads the event item out of the queue
        let res = xQueueReceive(EVENT_QUEUE.unwrap(), ptr::null_mut(), QUEUE_WAIT_TICKS);
        // If the event has the value 0, nothing happens. if it has a different value, the irq was triggered.
        res != 0
    }
}
