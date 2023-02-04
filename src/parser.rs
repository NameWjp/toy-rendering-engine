pub trait Parser {
    // 获取当前输入的值
    fn get_input(&self) -> &String;

    // 获取当前位置
    fn get_pos(&self) -> usize;

    // 设置当前位置
    fn set_pos(&mut self, pos: usize);

    // 消耗当前的字符
    fn consume_char(&mut self) -> char {
        let mut iter = self.get_input()[self.get_pos()..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.set_pos(self.get_pos() + next_pos);
        cur_char
    }

    // 获取下一个字符但是不消耗它
    fn next_char(&self) -> char {
        self.get_input()[self.get_pos()..].chars().next().unwrap()
    }

    // 消耗字符直到 test 返回 false
    fn consume_while<F>(&mut self, test: F) -> String where F: Fn(char) -> bool {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char())
        }
        result
    }

    // 消耗连续的空格
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    // 下一个字符是否以给定的字符串开头
    fn starts_with(&self, s: &str) -> bool {
        self.get_input()[self.get_pos()..].starts_with(s)
    }

    // 是否已经遍历完
    fn eof(&self) -> bool {
        self.get_pos() >= self.get_input().len()
    }
}