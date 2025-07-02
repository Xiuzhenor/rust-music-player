use color_eyre::Result;
use crossterm::event::{self, KeyCode, poll};
use derive_more::Display;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{
        Color,
        Style,
        Stylize,
        // palette::tailwind::{GREEN, SLATE},
    },
    symbols::border::THICK,
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};
use rodio::{Decoder, OutputStream, Sink};
use std::{
    env,
    fs::{File, read_dir},
    io::BufReader,
    path::PathBuf,
};

const SELECTED_STYLE: Style = Style::new().bg(Color::Rgb(56, 142, 60));

pub struct MusicPlayer {
    player_entrys: MusicPlayerFiles,
    play_state: PlayState,
    running: bool,
}

pub struct PlayState {
    play_mode: PlayMode,
    play_mode_changed: bool,
    volume: f32,
}

#[derive(Display)]
pub enum PlayMode {
    Nomal,
    Loop,
    PlayList,
    CurrentDir,
}

pub struct MusicPlayerFiles {
    current_dir: PathBuf,
    source_path: Option<PathBuf>,
    entry_list: FileList,
    entry_index: usize,
    play_list: Vec<PathBuf>,
    play_index: usize,
    play_list_changed: bool,
}

pub struct FileList {
    entrys: Vec<PathBuf>,
    state: ListState,
}

impl MusicPlayer {
    pub fn new() -> Self {
        let play_list: Vec<PathBuf> = Vec::new();
        let entry_list = FileList {
            entrys: Vec::new(),
            state: ListState::default(),
        };
        let current_dir = PathBuf::from(".");
        Self {
            player_entrys: MusicPlayerFiles {
                current_dir,
                source_path: None,
                entry_list,
                entry_index: 0,
                play_list,
                play_index: 0,
                play_list_changed: false,
            },
            play_state: PlayState {
                play_mode: PlayMode::Nomal,
                play_mode_changed: false,
                volume: 100f32,
            },
            running: true,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.initliaze()?;
        //创建句柄与音频轨道;creat a audio device handle and audio track
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        //程序主循环
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events(&sink)?;
        }
        Ok(())
    }

    fn initliaze(&mut self) -> Result<()> {
        self.read_files()?;
        self.select_first();
        Ok(())
    }

    fn handle_events(&mut self, sink: &Sink) -> Result<()> {
        if poll(std::time::Duration::from_millis(500))? {
            if let event::Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char(char) => {
                        if char == 'q' {
                            self.running = false;
                        }
                        if char == 'n' {
                            self.change_mode(PlayMode::Nomal);
                        }
                        if char == 'l' {
                            self.change_mode(PlayMode::Loop);
                        }
                        if char == 'p' {
                            self.change_mode(PlayMode::PlayList);
                        }
                        if char == 'c' {
                            self.change_mode(PlayMode::CurrentDir);
                        }
                        if char == '=' {
                            self.increase_volume(sink);
                        }
                        if char == '-' {
                            self.decrease_volume(sink);
                        }
                    }
                    //移动或选中列表项
                    KeyCode::Up => self.select_previous(),
                    KeyCode::Down => self.select_next(),
                    KeyCode::Enter => self.file_select(sink),
                    _ => (),
                }
            };
        }
        match self.play_state.play_mode {
            PlayMode::Loop => self.loop_mode(sink),
            PlayMode::PlayList => self.play_list_mode(sink),
            PlayMode::CurrentDir => self.currentdir_mode(sink),
            _ => (),
        }
        Ok(())
    }

    fn read_files(&mut self) -> Result<()> {
        self.player_entrys.entry_list.entrys.clear();
        //更改当前目录地址
        self.player_entrys.current_dir = env::current_dir().unwrap();
        //加入父目录到当前项目列表
        let parent_dir = self.player_entrys.current_dir.join(PathBuf::from(".."));
        self.player_entrys.entry_list.entrys.push(parent_dir);
        //读取当前文件夹下项目内容
        let mut entrys: Vec<PathBuf> = read_dir(env::current_dir().unwrap())?
            .map(|entry| entry.unwrap().path())
            .collect();
        //追加文件夹项目进入项目列表
        self.player_entrys.entry_list.entrys.append(&mut entrys);
        //选中第0个项目并改索引为0
        self.select_first();
        //当当前模式为当前目录模式时，重新将新目录内容加入歌单
        match self.play_state.play_mode {
            PlayMode::CurrentDir => self.currentfiles_to_playlist(),
            _ => (),
        }
        Ok(())
    }

    //选取文件进行操作
    fn file_select(&mut self, sink: &Sink) {
        //将路径索引作为实例读取
        let entry = self
            .player_entrys
            .entry_list
            .entrys
            .get(self.player_entrys.entry_index)
            .unwrap()
            .clone();
        if entry.is_file() {
            //播放音乐
            match read_files_to_stream(&entry) {
                Some(stream) => {
                    //将选中文件添加入播放源
                    self.player_entrys.source_path = Some(entry.clone());
                    let source = stream;
                    sink.stop();
                    sink.append(source);
                }

                None => (),
            }
        } else {
            //更改选中文件夹为当前文件夹
            // self.player_entrys.current_dir = entry;
            env::set_current_dir(entry).unwrap();
            //进入该文件夹将file_list转为当前文件夹内项目
            self.read_files().unwrap();
        }
    }

    fn select_first(&mut self) {
        self.player_entrys.entry_list.state.select(Some(0));
        self.player_entrys.entry_index = 0;
    }

    fn select_next(&mut self) {
        self.player_entrys.entry_list.state.select_next();
        if self.player_entrys.entry_index < self.player_entrys.entry_list.entrys.len() {
            self.player_entrys.entry_index += 1;
        }
    }

    fn select_previous(&mut self) {
        self.player_entrys.entry_list.state.select_previous();
        if self.player_entrys.entry_index != 0 {
            self.player_entrys.entry_index -= 1;
        }
    }
    //模式切换
    pub fn change_mode(&mut self, mode: PlayMode) {
        if matches!(&self.play_state.play_mode, _mode) {
            self.play_state.play_mode_changed = true;
            self.play_state.play_mode = mode;
        }
    }
    //控制模式行为
    fn loop_mode(&self, sink: &Sink) {
        if let Some(source_path) = &self.player_entrys.source_path {
            if sink.empty() {
                if let Some(source) = read_files_to_stream(&source_path) {
                    sink.append(source);
                };
            }
        }
    }

    fn play_list_mode(&mut self, sink: &Sink) {
        //判断播放列表当中有无元素，无元素直接无视该方法
        if let Some(_path) = self.player_entrys.play_list.get(0) {
            if self.play_state.play_mode_changed {
                self.play_state.play_mode_changed = false;
                //停止之前音乐的播放
                sink.stop();
            }
            //判断播放列表是否遭遇更改，若是则改为否，并重置播放列表索引
            if self.player_entrys.play_list_changed {
                self.player_entrys.play_list_changed = false;
                self.player_entrys.play_index = 0;
            }

            //查找播放索引处是否有歌曲，若无，则设置播放索引为零
            if let Some(source_path) = self
                .player_entrys
                .play_list
                .get(self.player_entrys.play_index)
            {
                //尝试读取该文件作为音频源
                if let Some(source) = read_files_to_stream(&source_path) {
                    sink.append(source);
                } else {
                    self.player_entrys.play_index += 1;
                    self.play_list_mode(sink);
                }
            } else {
                self.player_entrys.play_index = 0;
            }
        }
    }

    fn currentdir_mode(&mut self, sink: &Sink) {
        if self.play_state.play_mode_changed {
            self.currentfiles_to_playlist();
        }
        self.play_list_mode(sink);
    }

    fn currentfiles_to_playlist(&mut self) {
        //更改当前文件夹内内容为播放列表内容
        // self.player_entrys.play_list_changed = true;
        self.player_entrys.play_list.clear();
        self.player_entrys
            .entry_list
            .entrys
            .iter()
            .for_each(|entry| self.player_entrys.play_list.push(entry.clone()));
    }

    //音量调节
    fn increase_volume(&mut self, sink: &Sink) {
        if self.play_state.volume < 100f32 {
            self.play_state.volume += 5f32;
            self.set_volume(sink);
        }
    }

    fn decrease_volume(&mut self, sink: &Sink) {
        if self.play_state.volume > 0f32 {
            self.play_state.volume -= 5f32;
            self.set_volume(sink);
        }
    }

    fn set_volume(&self, sink: &Sink) {
        sink.set_volume(self.play_state.volume / 100f32);
    }
}

//读取文件
fn read_files_to_stream(file: &PathBuf) -> Option<Decoder<BufReader<File>>> {
    let file = File::open(file).ok()?;
    let source = Decoder::new(BufReader::new(file)).ok();
    source
}

impl Widget for &mut MusicPlayer {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Length(3)])
            .split(area);
        //设置标题
        let title = Line::from("R-Player").centered();
        //设置边框样式
        let block = Block::bordered().title(title).border_set(THICK);
        let bar_block = Block::bordered().border_set(THICK);
        //显示文件实例
        let file_list: Vec<ListItem> = self
            .player_entrys
            .entry_list
            .entrys
            .iter()
            .clone()
            .map(|entry| {
                ListItem::from(
                    entry
                        .strip_prefix(&self.player_entrys.current_dir)
                        .unwrap()
                        .to_string_lossy(),
                )
            })
            .collect();

        let list = List::new(file_list)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">");
        // .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(
            list,
            outer_layout[0],
            buf,
            &mut self.player_entrys.entry_list.state,
        );

        //显示状态栏
        let text = match self.play_state.play_mode {
            PlayMode::Nomal => "Nomal     ",
            PlayMode::Loop => "Loop      ",
            PlayMode::PlayList => "PlayList  ",
            PlayMode::CurrentDir => "CurrentDir",
        };
        let text = Line::from(vec![
            "PlayMode: ".yellow().into(),
            text.yellow().into(),
            "     volume： ".green().into(),
            self.play_state.volume.to_string().green().into(),
        ]);
        Paragraph::new(text)
            .block(bar_block)
            .render(outer_layout[1], buf);
    }
}
