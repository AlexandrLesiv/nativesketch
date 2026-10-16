use crate::{model::Model, provider::Provider};
use sketch_app::{HostEffect, Intent};
use sketch_core::application::{routing::*, *};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(1);
pub(crate) struct Fixture {
    pub model: Model,
    pub provider: Provider,
    pub directory: PathBuf,
}
impl Fixture {
    pub fn new() -> Self {
        let directory = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "sketch-gtk-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        Self {
            model: Model::new(),
            provider: Provider::new(),
            directory,
        }
    }
    pub fn stroke(&mut self) {
        self.model.down(40.0, 50.0, None, 100, 0).unwrap();
        self.model.motion(80.0, 80.0, None, 127).unwrap();
        self.model.up().unwrap();
    }
    pub fn action(&mut self, action: EditorAction) {
        self.model.owner.dispatch(Intent::Action(action)).unwrap();
    }
    pub fn file(&mut self, intent: FileIntent) {
        self.model.owner.dispatch(Intent::File(intent)).unwrap();
    }
    pub fn effect(&mut self) -> HostEffect {
        self.model.owner.effects.pop_front().expect("host effect")
    }
    pub fn reply(&mut self, token: EffectToken, decision: EffectDecision) {
        let context = self.model.owner.context().unwrap();
        self.model.owner.reply(context, token, decision).unwrap();
    }
    pub fn choose(&mut self, path: PathBuf) {
        let token = match self.effect() {
            HostEffect::ChooseDestination { token, .. } | HostEffect::ChooseSource(token) => token,
            effect => panic!("{effect:?}"),
        };
        let context = self.model.owner.context().unwrap();
        self.provider
            .supply(&mut self.model.owner, context, token, path)
            .unwrap();
        assert!(matches!(self.effect(), HostEffect::WorkQueued(_)));
    }
    pub fn drain(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.provider.poll(&mut self.model.owner).unwrap();
            if self.model.owner.app.stats().operations == 0 {
                break;
            }
            assert!(Instant::now() < deadline, "worker did not drain");
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    pub fn snapshot(&mut self) -> Vec<u8> {
        self.file(FileIntent::ExportCopy);
        let path = self.directory.join("copy.nsketch");
        self.choose(path.clone());
        self.drain();
        std::fs::read(path).unwrap()
    }
    pub fn open(&mut self, path: PathBuf) {
        self.file(FileIntent::Navigate(NavigationIntent::Open));
        if let Some(HostEffect::ConfirmUnsaved { token, .. }) =
            self.model.owner.effects.front().copied()
        {
            self.effect();
            self.reply(token, EffectDecision::Unsaved(UnsavedChoice::Discard));
        }
        self.choose(path);
        self.drain();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        self.model.owner.effects.clear();
        self.file(FileIntent::Navigate(NavigationIntent::Close));
        if let Some(HostEffect::ConfirmUnsaved { token, .. }) =
            self.model.owner.effects.front().copied()
        {
            self.effect();
            self.reply(token, EffectDecision::Unsaved(UnsavedChoice::Discard));
        }
        if let HostEffect::NavigationReady(token) = self.effect() {
            self.reply(token, EffectDecision::ConfirmClose);
        }
        assert!(matches!(self.effect(), HostEffect::CloseConfirmed));
        self.model
            .owner
            .app
            .detach_session(self.model.owner.session)
            .unwrap();
        self.drain();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !self.provider.stop_when_drained(&self.model.owner) {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(self.model.owner.app.stats().shutdown_ready());
        std::fs::remove_dir_all(&self.directory).unwrap();
    }
}
