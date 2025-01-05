use num_complex::Complex64;
use std::f64::consts::PI;
use chrono::prelude::*;
use plotters::{coord::Shift, prelude::*};
use wallpaper;
#[cfg(target_os = "windows")]
use {
    windows::System::UserProfile::LockScreen,
    windows::Storage::{StorageFile, IStorageFile},
    windows::core::{HSTRING, Interface}
};

const POINTS: usize = 10000;
const COLOR: RGBColor = RGBColor(255, 215, 0); // #FFD700
const WIDTH: u32 = 900;
const HEIGHT: u32 = 900;

fn date_polynom(idx: f64, date: &NaiveDate) -> f64 {
    let year = (date.year() - 2000) as f64;
    let month = date.month() as f64;
    let day = date.day() as f64;
    idx / month + idx.powi(2) / day + idx.powi(3) / year
}

fn date_polynom_exponential(idx: f64, date: &NaiveDate) -> Complex64 {
    Complex64::new(0.0, 2.0 * PI * date_polynom(idx, date)).exp()
}

fn get_current_partial_sums(date: &NaiveDate) -> Vec<Complex64> {
    (3..POINTS + 3)
        .map(|n| date_polynom_exponential(n as f64, date))
        .scan(Complex64::new(0.0, 0.0), |sum, x| {
            *sum += x;
            Some(*sum)
        })
        .collect()
}

fn plot_partial_sums(
    root: &DrawingArea<BitMapBackend, Shift>,
    partial_sums: &[Complex64],
) -> Result<(), Box<dyn std::error::Error>> {
    let min_x = partial_sums.iter().map(|c| c.re).fold(f64::INFINITY, f64::min);
    let max_x = partial_sums.iter().map(|c| c.re).fold(f64::NEG_INFINITY, f64::max);
    let min_y = partial_sums.iter().map(|c| c.im).fold(f64::INFINITY, f64::min);
    let max_y = partial_sums.iter().map(|c| c.im).fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(root)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    chart.configure_mesh().disable_mesh().disable_axes();

    chart.draw_series(LineSeries::new(
        partial_sums.iter().map(|c| (c.re, c.im)),
        COLOR.stroke_width(1),
    ))?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = Local::now().naive_local().date();
    let partial_sums = get_current_partial_sums(&now);

    let image_file = tempfile::NamedTempFile::new()?.into_temp_path().with_extension("png");
  
    let root = BitMapBackend::new(&image_file, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&BLACK)?;
  
    plot_partial_sums(&root, &partial_sums)?;
  
    root.present()?;
  
    wallpaper::set_from_path(&image_file.display().to_string())?;
    wallpaper::set_mode(wallpaper::Mode::Center)?;
  
    #[cfg(target_os = "windows")]
    {
        let image_path: String = image_file.display().to_string();
        let _ = set_windows_lock_screen(image_path)?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn set_windows_lock_screen(image_path: String) ->Result<(), Box<dyn std::error::Error>> {
    use std::error::Error;

    let file: StorageFile = StorageFile::GetFileFromPathAsync(&HSTRING::from(image_path))?.get()?;
    let file: IStorageFile = file.cast()?;
    let result = LockScreen::SetImageFileAsync(&file)?;
    match result.get() {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::<dyn Error>::from(e)),   
    }
}