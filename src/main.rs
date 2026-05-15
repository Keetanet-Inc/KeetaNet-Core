use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct ScreamControllerConfig {
    pub min_bitrate_bps: u32,      // Минимальная скорость (например, 100 Kbps)
    pub max_bitrate_bps: u32,      // Максимальная скорость (например, 50 Mbps)
    pub target_rtt_ms: u32,        // Целевой RTT (базовый порог задержки, например, 50ms)
    pub gain_factor: f32,          // Коэффициент агрессивности изменения скорости
}

impl Default for ScreamControllerConfig {
    fn default() -> Self {
        Self {
            min_bitrate_bps: 100_000,
            max_bitrate_bps: 50_000_000,
            target_rtt_ms: 50,
            gain_factor: 0.1,
        }
    }
}

pub struct ScreamController {
    config: ScreamControllerConfig,
    current_bitrate_bps: u32,
    last_update: Instant,
    sfixed_bytes_in_flight: usize,
}

impl ScreamController {
    pub fn new(config: ScreamControllerConfig) -> Self {
        let initial_bitrate = config.min_bitrate_bps;
        Self {
            config,
            current_bitrate_bps: initial_bitrate,
            last_update: Instant::now(),
            sfixed_bytes_in_flight: 0,
        }
    }

    /// Вызывается сетевым слоем при получении ACK/Feedback пакета от удаленной ноды.
    /// `current_rtt` - текущий замеренный RTT до ноды.
    /// `bytes_acknowledged` - объем успешно доставленных данных.
    /// `losses_detected` - были ли зафиксированы потери в этом цикле.
    pub fn on_feedback(&mut self, current_rtt: Duration, bytes_acknowledged: usize, losses_detected: bool) -> u32 {
        let now = Instant::now();
        let current_rtt_ms = current_rtt.as_millis() as u32;
        
        // 1. Защита от слишком частых апдейтов (лимитируем по интервалу)
        if now.duration_since(self.last_update) < Duration::from_millis(20) {
            return self.current_bitrate_bps;
        }
        self.last_update = now;

        // 2. Реакция на критические потери (Экстренное снижение скорости)
        if losses_detected {
            self.current_bitrate_bps = (self.current_bitrate_bps as f32 * 0.8) as u32; // -20% сразу
            self.clamp_bitrate();
            return self.current_bitrate_bps;
        }

        // 3. Математика SCReAM: Очередь рассчитывается как отклонение от целевого RTT
        // Если задержка растет, значит буферы (Network Queues) забиваются.
        let rtt_delta = current_rtt_ms as f32 - self.config.target_rtt_ms as f32;

        if rtt_delta > 0.0 {
            // Задержка выше нормы: плавно снижаем битрейт, чтобы разгрузить канал
            let reduction = (self.current_bitrate_bps as f32 * self.config.gain_factor * (rtt_delta / 100.0)) as u32;
            self.current_bitrate_bps = self.current_bitrate_bps.saturating_sub(reduction);
        } else {
            // Задержка в норме (буфер пуст): можно безопасно растить пропускную способность (L4S стиль)
            let increase = (bytes_acknowledged as f32 * self.config.gain_factor) as u32;
            self.current_bitrate_bps = self.current_bitrate_bps.saturating_add(increase);
        }

        self.clamp_bitrate();
        self.current_bitrate_bps
    }

    /// Вспомогательный метод удержания скорости в заданных бизнес-лимитах
    fn clamp_bitrate(&mut self) {
        if self.current_bitrate_bps < self.config.min_bitrate_bps {
            self.current_bitrate_bps = self.config.min_bitrate_bps;
        } else if self.current_bitrate_bps > self.config.max_bitrate_bps {
            self.current_bitrate_bps = self.config.max_bitrate_bps;
        }
    }

    /// Возвращает текущий разрешенный битрейт для кодировщика или планировщика пакетов
    pub fn get_target_bitrate(&self) -> u32 {
        self.current_bitrate_bps
    }
}

fn main() {
    // Пример инициализации для P2P слоя KeetaNet
    let config = ScreamControllerConfig {
        min_bitrate_bps: 500_000,      // 500 Kbps минимум для синхронизации метаданных
        max_bitrate_bps: 100_000_000,  // 100 Mbps максимум для обмена тяжелыми блоками
        target_rtt_ms: 30,             // Ожидаемый пинг в быстрых дата-центрах
        gain_factor: 0.15,
    };

    let mut controller = ScreamController::new(config);
    println!("Изначальный лимит сети: {} bps", controller.get_target_bitrate());

    // Симуляция 1: Идеальная сеть, данные доходят быстро (RTT 20ms < 30ms target)
    let next_bitrate = controller.on_feedback(Duration::from_millis(20), 65535, false);
    println!("Сеть чистая, увеличиваем лимит до: {} bps", next_bitrate);

    // Симуляция 2: Появился сетевой затор (RTT подскочил до 90ms)
    let next_bitrate = controller.on_feedback(Duration::from_millis(90), 65535, false);
    println!("Задержка выросла, SCReAM снижает скорость до: {} bps", next_bitrate);
}
