
struct Lev {
    dp: Vec<usize>,
}
impl Lev {
    fn new() -> Self {
        Self { dp: Vec::new() }
    }

    fn lev(&mut self, a: &str, b: &str) -> usize {
        let a = a.as_bytes();
        let b = b.as_bytes();
        let n = a.len();
        let m = b.len();
        self.dp.fill(0);
        self.dp.resize((n + 1) * (m + 1), 0);

        for i in 0..n + 1 {
            self.dp[i * (m + 1)] = i;
        }
        for j in 0..m + 1 {
            self.dp[j] = j;
        }
        for i in 1..n + 1 {
            for j in 1..m + 1 {
                if a[i - 1] == b[j - 1] {
                    self.dp[i * (m + 1) + j] = self.dp[(i - 1) * (m + 1) + (j - 1)];
                } else {
                    self.dp[i * (m + 1) + j] = self.dp[i * (m + 1) + (j - 1)]
                        .min(self.dp[(i - 1) * (m + 1) + j])
                        .min(self.dp[(i - 1) * (m + 1) + (j - 1)])
                        + 1;
                }
            }
        }
        return self.dp[self.dp.len() - 1];
    }
}