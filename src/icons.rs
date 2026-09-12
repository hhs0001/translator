//! App-specific icon names, resolved against the embedded asset bundle.

use gpui::SharedString;
use gpui_component::{Icon, IconNamed};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ico {
    ArrowDown,
    ArrowUp,
    Ban,
    Check,
    ChevronDown,
    CircleCheck,
    CircleX,
    Clock,
    Close,
    CloudUpload,
    Download,
    ExternalLink,
    File,
    FileText,
    Film,
    Folder,
    FolderOpen,
    Gauge,
    Globe,
    Info,
    Key,
    Languages,
    Layers,
    List,
    Loader,
    Minus,
    Moon,
    Pause,
    Pencil,
    Play,
    Plus,
    Refresh,
    Save,
    ScrollText,
    Search,
    Settings,
    Sparkles,
    Sun,
    Terminal,
    Trash,
    TriangleAlert,
    Wand,
    Zap,
}

impl IconNamed for Ico {
    fn path(self) -> SharedString {
        let path = match self {
            Self::ArrowDown => "icons/arrow-down.svg",
            Self::ArrowUp => "icons/arrow-up.svg",
            Self::Ban => "icons/ban.svg",
            Self::Check => "icons/check.svg",
            Self::ChevronDown => "icons/chevron-down.svg",
            Self::CircleCheck => "icons/circle-check.svg",
            Self::CircleX => "icons/circle-x.svg",
            Self::Clock => "icons/clock.svg",
            Self::Close => "icons/close.svg",
            Self::CloudUpload => "icons/cloud-upload.svg",
            Self::Download => "icons/download.svg",
            Self::ExternalLink => "icons/external-link.svg",
            Self::File => "icons/file.svg",
            Self::FileText => "icons/file-text.svg",
            Self::Film => "icons/film.svg",
            Self::Folder => "icons/folder.svg",
            Self::FolderOpen => "icons/folder-open.svg",
            Self::Gauge => "icons/gauge.svg",
            Self::Globe => "icons/globe.svg",
            Self::Info => "icons/info.svg",
            Self::Key => "icons/key.svg",
            Self::Languages => "icons/languages.svg",
            Self::Layers => "icons/layers.svg",
            Self::List => "icons/list.svg",
            Self::Loader => "icons/loader-circle.svg",
            Self::Minus => "icons/minus.svg",
            Self::Moon => "icons/moon.svg",
            Self::Pause => "icons/pause.svg",
            Self::Pencil => "icons/pencil.svg",
            Self::Play => "icons/play.svg",
            Self::Plus => "icons/plus.svg",
            Self::Refresh => "icons/refresh.svg",
            Self::Save => "icons/save.svg",
            Self::ScrollText => "icons/scroll-text.svg",
            Self::Search => "icons/search.svg",
            Self::Settings => "icons/settings.svg",
            Self::Sparkles => "icons/sparkles.svg",
            Self::Sun => "icons/sun.svg",
            Self::Terminal => "icons/terminal.svg",
            Self::Trash => "icons/delete.svg",
            Self::TriangleAlert => "icons/triangle-alert.svg",
            Self::Wand => "icons/wand.svg",
            Self::Zap => "icons/zap.svg",
        };
        SharedString::from(path)
    }
}

impl Ico {
    /// Builds a renderable icon.
    pub fn icon(self) -> Icon {
        Icon::new(self)
    }
}
