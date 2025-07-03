# rust-music-player

### 简介

这是一个由rust编写的tui音乐播放器，作者闲暇之余，目前功能比较简单，且目前未在windows上面进行任何测试，仅仅实现了选定歌曲进行单次/循环播放以及循环播放当前文件夹内所有可播放歌曲，目前支持格式待确定，不过mp3，flac和wav没发现什么问题，ogg有一些问题，但是我不太清楚原因，未来有待完善

### 构建安装

目前暂未提供发布包，感兴趣的小伙伴可以使用**cargo**自行构建

#### cargo安装
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 构建可执行文件
```shell
git clone https://github.com/Xiuzhenor/rust-music-player.git
cd rust-music-player.git
cargo build --release
```

最后添加别名到可执行文件路径或者环境变量即可

### app使用
打开软件时，会将当前目录读取并且列出当前目录当中所有项目，按下 **-** 和 **+** 键即可增减百分之五的音量

可以使用上键或者下键来移动光标，选定要操作的文件

当选定内容为文件时，按下Enter，app会尝试读取，读取失败则什么也不会发生(比如读取到不受支持的音频格式文件)

若选定内容为目录，则按下Enter，app会读取目录内容并且展示，以供操作，就像刚开始打开这个app一样

app暂时提供了3种可以使用的模式(列表播放模式暂未做好)，分别是单曲单次播放/单曲循环播放/当前目录下可以被播放的歌曲循环播放
按下n(Nomal模式),l(Loop模式),c(CurrentDir模式)即可分别进入
