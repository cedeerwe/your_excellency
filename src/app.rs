use egui::Color32;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GameState {
    excellency: Excellency,
    enemies: Vec<Enemy>,
    enemy_spawner: EnemySpawner,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct EnemySpawner {
    timer: Timer,
    maximum_hp: f32,
    speed: f32,
    damage: f32,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Timer {
    total: f32,
    remaining: f32,
    has_just_finished: bool,
    one_shot: bool,
    paused: bool,
}

impl Timer {
    pub fn new(total: f32) -> Self {
        Self {
            total,
            remaining: total,
            has_just_finished: false,
            one_shot: false,
            paused: false,
        }
    }

    pub fn tick(&mut self, delta: f32) {
        if !self.paused {
            self.remaining -= delta;
            if self.remaining <= 0. {
                self.has_just_finished = true;
                self.remaining += self.total; // Necessary because of the case when we aren't exactly 0
                if self.one_shot {
                    self.paused = true
                }
            } else {
                self.has_just_finished = false
            }
        }
    }

    pub fn has_just_finished(&self) -> bool {
        self.has_just_finished
    }

    // pub fn pause(&mut self) {
    //     self.paused = true
    // }

    // pub fn unpause(&mut self) {
    //     self.paused = false
    // }
}

impl GameState {
    pub fn tick(&mut self, ctx: &egui::Context) {
        let delta = ctx.input(|i| i.stable_dt);

        let mut enemies = vec![];

        for enemy in self.enemies.iter_mut() {
            match enemy.tick(delta) {
                EnemyAfterTick::Normal => enemies.push(enemy.clone()), // TODO: Clone isn't strictly necessary here
                EnemyAfterTick::ReachedExcellency => self.excellency.hp.take_damage(enemy.damage),
            }
        }

        self.enemies = enemies;

        self.enemy_spawner.timer.tick(delta);
        if self.enemy_spawner.timer.has_just_finished() {
            self.enemies.push(Enemy {
                hp: HitPoints::new_full(self.enemy_spawner.maximum_hp),
                damage: self.enemy_spawner.damage,
                speed: self.enemy_spawner.speed,
                distance: Distance::start(),
            })
        }

        self.enemies.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .expect("Compared two f32's")
        })
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Excellency {
    hp: HitPoints,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct Enemy {
    hp: HitPoints,
    damage: f32,
    speed: f32,
    distance: Distance,
}

impl Enemy {
    pub fn tick(&mut self, delta: f32) -> EnemyAfterTick {
        self.distance.0 -= delta * self.speed;
        match self.distance.0 > 0. {
            true => EnemyAfterTick::Normal,
            false => EnemyAfterTick::ReachedExcellency,
        }
    }
}

enum EnemyAfterTick {
    Normal,
    ReachedExcellency,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Clone)]
struct Distance(f32);

impl Distance {
    pub fn start() -> Self {
        Self(100.)
    }

    pub fn as_progress_bar(&self) -> egui::ProgressBar {
        egui::ProgressBar::new(self.0 / 100.).show_percentage()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct HitPoints {
    maximum: f32,
    current: f32,
}

impl HitPoints {
    pub fn new_full(maximum: f32) -> Self {
        Self {
            maximum,
            current: maximum,
        }
    }

    pub fn as_progress_bar(&self) -> egui::ProgressBar {
        egui::ProgressBar::new(self.current / self.maximum)
            .text(format!("{}/{}", self.current, self.maximum))
            .fill(Color32::DARK_RED)
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.current -= damage;
    }

    pub fn reset(&mut self) {
        self.current = self.maximum;
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            excellency: Excellency {
                hp: HitPoints::new_full(100.),
            },
            enemy_spawner: EnemySpawner {
                timer: Timer::new(1.),
                maximum_hp: 10.,
                speed: 5.,
                damage: 2.,
            },
            enemies: vec![],
        }
    }
}

impl GameState {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        // TODO: Commented out persistence for now
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }
}

impl eframe::App for GameState {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        self.tick(ctx);

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.heading("Enemy spawner");
            ui.horizontal(|ui| {
                ui.label("Damage:");
                ui.add(egui::Slider::new(&mut self.enemy_spawner.damage, 0. ..=20.));
            });
            ui.horizontal(|ui| {
                ui.label("Maximum HP:");
                ui.add(egui::Slider::new(
                    &mut self.enemy_spawner.maximum_hp,
                    0. ..=100.,
                ));
            });
            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(egui::Slider::new(&mut self.enemy_spawner.speed, 0. ..=20.));
            });

            ui.separator();
            ui.heading("Enemies");

            egui::ScrollArea::vertical().show(ui, |ui| {
                for enemy in self.enemies.iter() {
                    ui.horizontal(|ui| {
                        ui.label("Distance:");
                        ui.add(enemy.distance.as_progress_bar());
                    });
                    ui.horizontal(|ui| {
                        ui.label("HP:");
                        ui.add(enemy.hp.as_progress_bar());
                    });
                    ui.label(format!("Damage: {}", enemy.damage));
                    ui.label(format!("Speed: {}", enemy.speed));
                    ui.separator();
                }
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Player stuff");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("HP:");
                ui.add(self.excellency.hp.as_progress_bar());
            });
            if ui.button("Reset HP").clicked() {
                self.excellency.hp.reset()
            }
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(16)) // ~60fps
    }
}
