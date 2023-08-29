pub struct UTF32String {
    pub inner: Vec<char>,
}
impl UTF32String {
    pub fn insert(&mut self, index: usize, element: char) {
        self.inner.insert(index, element)
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn remove(&mut self, i: usize) {
        self.inner.remove(i);
    }
    pub fn trimmed(&self) -> &[char] {
        let s = self
            .inner
            .iter()
            .position(|c| !c.is_ascii_whitespace())
            .unwrap_or(0);
        let e = self
            .inner
            .iter()
            .rposition(|c| !c.is_ascii_whitespace())
            .unwrap_or(self.inner.len());
        &self.inner[s..e]
    }

    pub fn starts_with(&self, s: &str) -> bool {
        for (a, b) in std::iter::zip(&self.inner, s.chars()) {
            if a != &b {
                return false;
            }
        }
		return true;
    }
}

impl std::fmt::Display for UTF32String {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.inner {
            write!(f, "{c}")?;
        }
        Ok(())
    }
}
