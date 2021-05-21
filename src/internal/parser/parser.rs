pub enum Loop {
    Continue,
    Stop,
}

pub trait Parser {
    type Parsed;
    fn parse(mut self) -> Self::Parsed where Self: Sized {
        let mut keep = Loop::Continue;

        while let Loop::Continue = keep {
            keep = self.next();
        }

        self.get_parsed()
    }

    #[inline(always)]
    fn next(&mut self) -> Loop {
        if self.get_index() == self.get_original_len() {
            return Loop::Stop;
        }

        self.parse_next();

        Loop::Continue
    }

    fn get_index(&self) -> usize;
    fn get_original_len(&self) -> usize;
    fn get_parsed(self) -> Self::Parsed;
    fn parse_next(&mut self);
}