use crate::{templates::render, views::View};
use minijinja::context;
use minijinja_autoreload::AutoReloader;

pub struct Info {
    sys: sysinfo::System,
}

impl Info {
    pub fn new() -> Self {
        let sys = sysinfo::System::new_all();
        Self { sys }
    }

    fn uptime(&self) -> String {
        let mut uptime_secs = sysinfo::System::uptime();
        let uptime_days = uptime_secs / 60 / 60 / 24;
        uptime_secs -= uptime_days * (60 * 60 * 24);
        let uptime_hours = uptime_secs / 60 / 60;
        uptime_secs -= uptime_hours * (60 * 60);
        let uptime_mins = uptime_secs / 60;
        format!("{uptime_days} days, {uptime_hours} hours, {uptime_mins} minutes")
    }

    fn processor(&self) -> String {
        let cpu = self.sys.cpus().first().unwrap();
        let cpu_brand = cpu
            .brand()
            .replace("(R)", "®")
            .replace("(TM)", "™")
            .replace("CPU", "");
        let cpu_name = cpu.name().replace("1", "").replace("cpu0", "");
        let cores = sysinfo::System::physical_core_count().unwrap();
        format!("{cpu_brand} {cpu_name} ({cores} cores)")
    }

    fn computer(&self) -> String {
        let product_name = sysinfo::Product::name().unwrap();
        let vendor_name = sysinfo::Product::vendor_name().unwrap();
        format!("{vendor_name} {product_name}")
    }
}

impl View for Info {
    fn render(&self, reloader: &AutoReloader) -> anyhow::Result<String> {
        let computer = self.computer();
        let processor = self.processor();
        let memory = self.sys.total_memory() / 2u64.pow(30);
        let uptime = self.uptime();
        let html = render(
            reloader,
            "views/info",
            context! {
                computer,
                processor,
                memory,
                uptime,
            },
        )?;

        Ok(html)
    }
}
