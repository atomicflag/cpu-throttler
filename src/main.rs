use std::{fs, io, path::PathBuf, thread, time::Duration};

static MAX_PERF_PCT: &str = "/sys/devices/system/cpu/intel_pstate/max_perf_pct";
const OPTIMAL_CPU_PCT: f32 = 72.; // Try to keep MAX_PERF_PCT close to this value when throttling
const TEMP_THRESHOLD: f32 = 75.; // Start throttling when the temperature reaches this value
const MAX_TEMP: f32 = 80.; // Maximum throttling at this temperature
const MAX_CPU: f32 = 100.; // Maximum value for MAX_PERF_PCT (value below TEMP_THRESHOLD)
const MIN_CPU: f32 = 66.; // Minimum value for MAX_PERF_PCT (value at MAX_TEMP and above)
const THROTTLE_CPU: f32 = 75.; // Starting value for MAX_PERF_PCT (value at TEMP_THRESHOLD)

/*
                        TEMP_THRESHOLD
                        |       MAX_TEMP
                        |       |
Temperature    <75      75      80      100
-------------------------------------------
Throttle       100      75      66      66
                |       |       |
                |       |       MIN_CPU
                |       THROTTLE_CPU
                MAX_CPU
*/

fn cpu_sensor() -> io::Result<PathBuf> {
    for d in fs::read_dir("/sys/class/hwmon")? {
        let d = d?.path();
        if fs::read_to_string(d.join("name"))? == "coretemp\n" {
            return Ok(d.join("temp1_input"));
        }
    }
    Err(io::Error::other("Sensor not found"))
}

fn read_temp(path: &PathBuf) -> io::Result<f32> {
    fs::read_to_string(path)?
        .trim_end()
        .parse()
        .map(|v: f32| v / 1000.)
        .or(Err(io::Error::other("Can't parse sensor data")))
}

fn main() -> io::Result<()> {
    let sensor = cpu_sensor()?;
    let mut cpu_pct: f32 = MAX_CPU;
    let mut last_cpu_pct: u32 = MAX_CPU as u32;
    loop {
        let t = read_temp(&sensor)?;
        if t >= TEMP_THRESHOLD {
            let mut x = (t.min(MAX_TEMP) - TEMP_THRESHOLD) / 5.;
            x = (THROTTLE_CPU - MIN_CPU) * (1. - x) + MIN_CPU;
            cpu_pct = cpu_pct.min(x);
        }
        cpu_pct += ((cpu_pct - OPTIMAL_CPU_PCT).abs() / 20.).max(0.05);
        cpu_pct = cpu_pct.min(MAX_CPU);
        if cpu_pct as u32 != last_cpu_pct {
            fs::write(MAX_PERF_PCT, (cpu_pct as u32).to_string())?;
            last_cpu_pct = cpu_pct as u32;
        }
        thread::sleep(Duration::from_millis(500));
    }
}
