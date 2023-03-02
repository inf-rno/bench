use std::fmt::Display;
use std::io;
use std::io::Write;
use std::ops::AddAssign;

use hdrhistogram::Histogram;

#[derive(Debug, Clone)]
pub struct HDR(hdrhistogram::Histogram<u64>);

impl HDR {
    pub fn new() -> Self {
        HDR(Histogram::<u64>::new_with_bounds(10, 60000000, 3).unwrap())
    }

    pub fn p99(&self) -> u64 {
        self.0.value_at_quantile(0.99)
    }

    pub fn percentiles<W: Write>(&self, mut writer: W) -> Result<(), io::Error> {
        let quantile_precision = 6;
        let ticks_per_half = 10;
        let hist = &self.0;

        writer.write_all(
            format!(
                "{:>12} {:>quantile_precision$} {:>10} {:>14}\n\n",
                "Value",
                "Percentile",
                "TotalCount",
                "1/(1-Percentile)",
                quantile_precision = quantile_precision + 2 // + 2 from leading "0." for numbers
            )
            .as_ref(),
        )?;
        let mut sum = 0;
        for v in hist.iter_quantiles(ticks_per_half) {
            sum += v.count_since_last_iteration();
            if v.quantile_iterated_to() < 1.0 {
                writer.write_all(
                    format!(
                        "{:12} {:1.*} {:10} {:14.2}\n",
                        v.value_iterated_to() as f64 / 1000.0,
                        quantile_precision,
                        v.quantile_iterated_to(),
                        sum,
                        1_f64 / (1_f64 - v.quantile_iterated_to())
                    )
                    .as_ref(),
                )?;
            } else {
                writer.write_all(
                    format!(
                        "{:12} {:1.*} {:10} {:>14}\n",
                        v.value_iterated_to() as f64 / 1000.0,
                        quantile_precision,
                        v.quantile_iterated_to(),
                        sum,
                        "∞"
                    )
                    .as_ref(),
                )?;
            }
        }

        fn write_extra_data<T1: Display, T2: Display, W: Write>(
            writer: &mut W,
            label1: &str,
            data1: T1,
            label2: &str,
            data2: T2,
        ) -> Result<(), io::Error> {
            writer.write_all(
                format!(
                    "#[{:10} = {:12.2}, {:14} = {:12.2}]\n",
                    label1, data1, label2, data2
                )
                .as_ref(),
            )
        }

        write_extra_data(
            &mut writer,
            "Mean",
            hist.mean() / 1000.0,
            "StdDeviation",
            hist.stdev() / 1000.0,
        )?;
        write_extra_data(
            &mut writer,
            "Max",
            hist.max() as f64 / 1000.0,
            "Total count",
            hist.len(),
        )?;
        write_extra_data(
            &mut writer,
            "Buckets",
            hist.buckets(),
            "SubBuckets",
            hist.distinct_values(),
        )?;

        Ok(())
    }
}

impl AddAssign<u64> for HDR {
    fn add_assign(&mut self, value: u64) {
        self.0 += value;
    }
}

impl Default for HDR {
    fn default() -> Self {
        HDR::new()
    }
}
