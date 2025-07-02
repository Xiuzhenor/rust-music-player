use crate::MusicPlayer;
use crossterm::{
    cursor, execute,
    style::{self, Color, Print},
    terminal,
};

pub trait CursorMove {
    fn move_row(&self, row: u16);
}

impl CursorMove for MusicPlayer {
    //移动光标与选中
    fn move_row(&self, target_row: u16) {
        let index = target_row as usize;
        let top_line_text_index = self.top_line_text_index as usize;
        match &self.file_list.get(index + top_line_text_index) {
            Some(path) => {
                //获取光标所在行
                let (_, cursor_line) = cursor::position().unwrap();
                let current_line_text = self
                    .file_list
                    .get((cursor_line + self.top_line_text_index) as usize)
                    .unwrap();
                //重新打印旧行(颜色改为未被选中实例颜色)
                execute!(
                    &self.stdout,
                    terminal::Clear(terminal::ClearType::CurrentLine),
                    style::ResetColor,
                    Print("\r"),
                    Print(
                        current_line_text
                            .strip_prefix(&self.current_dir)
                            .unwrap_or(&current_line_text)
                            .display()
                    ),
                    //移动光标并打印新行(将实例设为选中状态颜色)
                    cursor::MoveToRow(target_row),
                    style::SetForegroundColor(Color::Yellow),
                    terminal::Clear(terminal::ClearType::CurrentLine),
                    Print("\r"),
                    Print(
                        path.strip_prefix(&self.current_dir)
                            .unwrap_or(&path)
                            .display()
                    )
                )
                .unwrap();
            }
            None => (),
        }
    }
}
