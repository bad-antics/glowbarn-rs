//! Statistical analysis and hypothesis testing

use serde::{Deserialize, Serialize};

/// Statistical summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatisticalSummary {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub mode: Option<f64>,
    pub std_dev: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub range: f64,
    pub q1: f64,
    pub q3: f64,
    pub iqr: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub coefficient_of_variation: f64,
}

/// Statistical analyzer
pub struct StatisticalAnalyzer;

impl StatisticalAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn summarize(&self, data: &[f64]) -> StatisticalSummary {
        if data.is_empty() {
            return StatisticalSummary::default();
        }
        
        let count = data.len();
        let mean = data.iter().sum::<f64>() / count as f64;
        
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median = if count % 2 == 0 {
            (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
        } else {
            sorted[count / 2]
        };
        
        let min = sorted[0];
        let max = sorted[count - 1];
        let range = max - min;
        
        let q1 = self.percentile(&sorted, 25.0);
        let q3 = self.percentile(&sorted, 75.0);
        let iqr = q3 - q1;
        
        let variance = if count > 1 {
            data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (count - 1) as f64
        } else {
            0.0
        };
        let std_dev = variance.sqrt();
        
        let (skewness, kurtosis) = if std_dev > 1e-10 && count > 3 {
            let n = count as f64;
            let skew = data.iter()
                .map(|&x| ((x - mean) / std_dev).powi(3))
                .sum::<f64>() * n / ((n - 1.0) * (n - 2.0));
            
            let kurt = data.iter()
                .map(|&x| ((x - mean) / std_dev).powi(4))
                .sum::<f64>() * n * (n + 1.0) / ((n - 1.0) * (n - 2.0) * (n - 3.0))
                - 3.0 * (n - 1.0).powi(2) / ((n - 2.0) * (n - 3.0));
            
            (skew, kurt)
        } else {
            (0.0, 0.0)
        };
        
        let coefficient_of_variation = if mean.abs() > 1e-10 {
            std_dev / mean.abs()
        } else {
            0.0
        };
        
        let mode = self.calculate_mode(&sorted);
        
        StatisticalSummary {
            count,
            mean,
            median,
            mode,
            std_dev,
            variance,
            min,
            max,
            range,
            q1,
            q3,
            iqr,
            skewness,
            kurtosis,
            coefficient_of_variation,
        }
    }
    
    fn percentile(&self, sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let k = (p / 100.0 * (sorted.len() - 1) as f64);
        let f = k.floor() as usize;
        let c = k.ceil() as usize;
        
        if f == c || c >= sorted.len() {
            sorted[f.min(sorted.len() - 1)]
        } else {
            sorted[f] + (sorted[c] - sorted[f]) * (k - f as f64)
        }
    }
    
    fn calculate_mode(&self, sorted: &[f64]) -> Option<f64> {
        if sorted.is_empty() {
            return None;
        }
        
        // Bin the data and find most common bin
        let n_bins = (sorted.len() as f64).sqrt() as usize;
        if n_bins < 3 {
            return None;
        }
        
        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let bin_width = (max - min) / n_bins as f64;
        
        if bin_width < 1e-10 {
            return Some(min);
        }
        
        let mut bins = vec![0usize; n_bins];
        for &x in sorted {
            let bin = ((x - min) / bin_width) as usize;
            let bin = bin.min(n_bins - 1);
            bins[bin] += 1;
        }
        
        let (max_bin, _) = bins.iter().enumerate().max_by_key(|(_, &c)| c)?;
        Some(min + (max_bin as f64 + 0.5) * bin_width)
    }
    
    /// Welch's t-test for comparing two samples
    pub fn welch_t_test(&self, sample1: &[f64], sample2: &[f64]) -> TTestResult {
        let n1 = sample1.len() as f64;
        let n2 = sample2.len() as f64;
        
        if n1 < 2.0 || n2 < 2.0 {
            return TTestResult {
                t_statistic: 0.0,
                p_value: 1.0,
                degrees_of_freedom: 0.0,
                significant: false,
            };
        }
        
        let mean1 = sample1.iter().sum::<f64>() / n1;
        let mean2 = sample2.iter().sum::<f64>() / n2;
        
        let var1 = sample1.iter().map(|&x| (x - mean1).powi(2)).sum::<f64>() / (n1 - 1.0);
        let var2 = sample2.iter().map(|&x| (x - mean2).powi(2)).sum::<f64>() / (n2 - 1.0);
        
        let se = (var1 / n1 + var2 / n2).sqrt();
        
        if se < 1e-10 {
            return TTestResult {
                t_statistic: 0.0,
                p_value: 1.0,
                degrees_of_freedom: n1 + n2 - 2.0,
                significant: false,
            };
        }
        
        let t = (mean1 - mean2) / se;
        
        // Welch-Satterthwaite degrees of freedom
        let df = (var1 / n1 + var2 / n2).powi(2) / (
            (var1 / n1).powi(2) / (n1 - 1.0) + (var2 / n2).powi(2) / (n2 - 1.0)
        );
        
        // Approximate p-value using Student's t distribution
        let p_value = self.t_distribution_p_value(t.abs(), df);
        
        TTestResult {
            t_statistic: t,
            p_value,
            degrees_of_freedom: df,
            significant: p_value < 0.05,
        }
    }
    
    fn t_distribution_p_value(&self, t: f64, df: f64) -> f64 {
        // Approximation using normal distribution for large df
        if df > 30.0 {
            return 2.0 * self.normal_cdf(-t.abs());
        }
        
        // Beta function approximation for small df
        let x = df / (df + t * t);
        let p = 0.5 * self.regularized_beta(x, df / 2.0, 0.5);
        2.0 * p
    }
    
    fn normal_cdf(&self, x: f64) -> f64 {
        0.5 * (1.0 + self.erf(x / std::f64::consts::SQRT_2))
    }
    
    fn erf(&self, x: f64) -> f64 {
        // Approximation
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;
        
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();
        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
        
        sign * y
    }
    
    fn regularized_beta(&self, x: f64, a: f64, b: f64) -> f64 {
        // Simplified approximation
        if x <= 0.0 { return 0.0; }
        if x >= 1.0 { return 1.0; }
        
        // Use continued fraction for better accuracy
        let mut result = x.powf(a) * (1.0 - x).powf(b) / (a * self.beta(a, b));
        
        let mut sum = 1.0;
        let mut term = 1.0;
        for n in 1..100 {
            term *= (a + b + n as f64 - 1.0) * x / (a + n as f64);
            sum += term;
            if term.abs() < 1e-10 {
                break;
            }
        }
        
        result * sum
    }
    
    fn beta(&self, a: f64, b: f64) -> f64 {
        (self.gamma_ln(a) + self.gamma_ln(b) - self.gamma_ln(a + b)).exp()
    }
    
    fn gamma_ln(&self, x: f64) -> f64 {
        // Lanczos approximation
        let g = 7.0;
        let c = [
            0.99999999999980993,
            676.5203681218851,
            -1259.1392167224028,
            771.32342877765313,
            -176.61502916214059,
            12.507343278686905,
            -0.13857109526572012,
            9.9843695780195716e-6,
            1.5056327351493116e-7,
        ];
        
        let x = x - 1.0;
        let mut sum = c[0];
        for i in 1..9 {
            sum += c[i] / (x + i as f64);
        }
        
        let t = x + g + 0.5;
        0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + sum.ln()
    }
    
    /// Mann-Whitney U test (non-parametric)
    pub fn mann_whitney_test(&self, sample1: &[f64], sample2: &[f64]) -> UTestResult {
        let n1 = sample1.len();
        let n2 = sample2.len();
        
        if n1 < 3 || n2 < 3 {
            return UTestResult {
                u_statistic: 0.0,
                p_value: 1.0,
                significant: false,
            };
        }
        
        // Combine and rank
        let mut combined: Vec<(f64, usize)> = sample1.iter()
            .map(|&x| (x, 0usize))
            .chain(sample2.iter().map(|&x| (x, 1usize)))
            .collect();
        combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        // Assign ranks
        let mut ranks = vec![0.0; combined.len()];
        let mut i = 0;
        while i < combined.len() {
            let mut j = i;
            while j < combined.len() && (combined[j].0 - combined[i].0).abs() < 1e-10 {
                j += 1;
            }
            let avg_rank = (i + j + 1) as f64 / 2.0 + 0.5;
            for k in i..j {
                ranks[k] = avg_rank;
            }
            i = j;
        }
        
        // Sum ranks for sample 1
        let r1: f64 = combined.iter()
            .zip(ranks.iter())
            .filter(|((_, group), _)| *group == 0)
            .map(|(_, &r)| r)
            .sum();
        
        let u1 = r1 - (n1 * (n1 + 1)) as f64 / 2.0;
        let u2 = (n1 * n2) as f64 - u1;
        let u = u1.min(u2);
        
        // Normal approximation for p-value
        let mean_u = (n1 * n2) as f64 / 2.0;
        let std_u = ((n1 * n2 * (n1 + n2 + 1)) as f64 / 12.0).sqrt();
        
        let z = if std_u > 1e-10 { (u - mean_u) / std_u } else { 0.0 };
        let p_value = 2.0 * self.normal_cdf(-z.abs());
        
        UTestResult {
            u_statistic: u,
            p_value,
            significant: p_value < 0.05,
        }
    }
    
    /// Kolmogorov-Smirnov test for distribution comparison
    pub fn ks_test(&self, sample: &[f64], theoretical_cdf: impl Fn(f64) -> f64) -> KSTestResult {
        if sample.is_empty() {
            return KSTestResult {
                d_statistic: 0.0,
                p_value: 1.0,
                significant: false,
            };
        }
        
        let n = sample.len();
        let mut sorted = sample.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mut d_max: f64 = 0.0;
        
        for (i, &x) in sorted.iter().enumerate() {
            let f_empirical = (i + 1) as f64 / n as f64;
            let f_empirical_prev = i as f64 / n as f64;
            let f_theoretical = theoretical_cdf(x);
            
            d_max = d_max
                .max((f_empirical - f_theoretical).abs())
                .max((f_empirical_prev - f_theoretical).abs());
        }
        
        // Kolmogorov distribution approximation for p-value
        let sqrt_n = (n as f64).sqrt();
        let p_value = self.kolmogorov_p_value(d_max * sqrt_n);
        
        KSTestResult {
            d_statistic: d_max,
            p_value,
            significant: p_value < 0.05,
        }
    }
    
    fn kolmogorov_p_value(&self, z: f64) -> f64 {
        if z < 0.27 {
            return 1.0;
        }
        if z > 3.5 {
            return 0.0;
        }
        
        // Approximation
        let mut sum = 0.0;
        for k in 1..100 {
            let term = (-2.0 * (k as f64).powi(2) * z * z).exp();
            sum += if k % 2 == 1 { term } else { -term };
            if term.abs() < 1e-10 {
                break;
            }
        }
        
        2.0 * sum
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTestResult {
    pub t_statistic: f64,
    pub p_value: f64,
    pub degrees_of_freedom: f64,
    pub significant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTestResult {
    pub u_statistic: f64,
    pub p_value: f64,
    pub significant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KSTestResult {
    pub d_statistic: f64,
    pub p_value: f64,
    pub significant: bool,
}
