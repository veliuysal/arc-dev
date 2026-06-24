use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum PackageManager {
    #[default]
    Npm,
    Pnpm,
    Yarn,
}

impl PackageManager {
    pub fn command(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
        }
    }
}
