pub enum Loop {
    Continue,
    Stop,
}

pub trait Parser {
    fn parse(mut self) -> Vec<u8> where Self: Sized {
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
    fn get_parsed(self) -> Vec<u8>;
    fn parse_next(&mut self);
}