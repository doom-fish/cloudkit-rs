use crate::private::CKNotificationInfoPayload;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKNotificationInfo {
    alert_body: Option<String>,
    alert_localization_key: Option<String>,
    alert_localization_args: Option<Vec<String>>,
    title: Option<String>,
    title_localization_key: Option<String>,
    title_localization_args: Option<Vec<String>>,
    subtitle: Option<String>,
    subtitle_localization_key: Option<String>,
    subtitle_localization_args: Option<Vec<String>>,
    alert_action_localization_key: Option<String>,
    alert_launch_image: Option<String>,
    sound_name: Option<String>,
    desired_keys: Option<Vec<String>>,
    should_badge: bool,
    should_send_content_available: bool,
    should_send_mutable_content: bool,
    category: Option<String>,
    collapse_id_key: Option<String>,
}

impl Default for CKNotificationInfo {
    fn default() -> Self {
        Self {
            alert_body: None,
            alert_localization_key: None,
            alert_localization_args: None,
            title: None,
            title_localization_key: None,
            title_localization_args: None,
            subtitle: None,
            subtitle_localization_key: None,
            subtitle_localization_args: None,
            alert_action_localization_key: None,
            alert_launch_image: None,
            sound_name: None,
            desired_keys: None,
            should_badge: false,
            should_send_content_available: true,
            should_send_mutable_content: false,
            category: None,
            collapse_id_key: None,
        }
    }
}

impl CKNotificationInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alert_body(&self) -> Option<&str> {
        self.alert_body.as_deref()
    }

    pub fn alert_localization_key(&self) -> Option<&str> {
        self.alert_localization_key.as_deref()
    }

    pub fn alert_localization_args(&self) -> Option<&[String]> {
        self.alert_localization_args.as_deref()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn title_localization_key(&self) -> Option<&str> {
        self.title_localization_key.as_deref()
    }

    pub fn title_localization_args(&self) -> Option<&[String]> {
        self.title_localization_args.as_deref()
    }

    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub fn subtitle_localization_key(&self) -> Option<&str> {
        self.subtitle_localization_key.as_deref()
    }

    pub fn subtitle_localization_args(&self) -> Option<&[String]> {
        self.subtitle_localization_args.as_deref()
    }

    pub fn alert_action_localization_key(&self) -> Option<&str> {
        self.alert_action_localization_key.as_deref()
    }

    pub fn alert_launch_image(&self) -> Option<&str> {
        self.alert_launch_image.as_deref()
    }

    pub fn sound_name(&self) -> Option<&str> {
        self.sound_name.as_deref()
    }

    pub fn desired_keys(&self) -> Option<&[String]> {
        self.desired_keys.as_deref()
    }

    pub const fn should_badge(&self) -> bool {
        self.should_badge
    }

    pub const fn should_send_content_available(&self) -> bool {
        self.should_send_content_available
    }

    pub const fn should_send_mutable_content(&self) -> bool {
        self.should_send_mutable_content
    }

    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    pub fn collapse_id_key(&self) -> Option<&str> {
        self.collapse_id_key.as_deref()
    }

    pub fn with_alert_body(mut self, alert_body: impl Into<String>) -> Self {
        self.alert_body = Some(alert_body.into());
        self
    }

    pub fn with_alert_localization_key(mut self, key: impl Into<String>) -> Self {
        self.alert_localization_key = Some(key.into());
        self
    }

    pub fn with_alert_localization_args(mut self, args: Vec<String>) -> Self {
        self.alert_localization_args = Some(args);
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_title_localization_key(mut self, key: impl Into<String>) -> Self {
        self.title_localization_key = Some(key.into());
        self
    }

    pub fn with_title_localization_args(mut self, args: Vec<String>) -> Self {
        self.title_localization_args = Some(args);
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn with_subtitle_localization_key(mut self, key: impl Into<String>) -> Self {
        self.subtitle_localization_key = Some(key.into());
        self
    }

    pub fn with_subtitle_localization_args(mut self, args: Vec<String>) -> Self {
        self.subtitle_localization_args = Some(args);
        self
    }

    pub fn with_alert_action_localization_key(mut self, key: impl Into<String>) -> Self {
        self.alert_action_localization_key = Some(key.into());
        self
    }

    pub fn with_alert_launch_image(mut self, image: impl Into<String>) -> Self {
        self.alert_launch_image = Some(image.into());
        self
    }

    pub fn with_sound_name(mut self, sound_name: impl Into<String>) -> Self {
        self.sound_name = Some(sound_name.into());
        self
    }

    pub fn with_desired_keys(mut self, desired_keys: Vec<String>) -> Self {
        self.desired_keys = Some(desired_keys);
        self
    }

    pub fn with_should_badge(mut self, should_badge: bool) -> Self {
        self.should_badge = should_badge;
        self
    }

    pub fn with_content_available(mut self, should_send_content_available: bool) -> Self {
        self.should_send_content_available = should_send_content_available;
        self
    }

    pub fn with_mutable_content(mut self, should_send_mutable_content: bool) -> Self {
        self.should_send_mutable_content = should_send_mutable_content;
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn with_collapse_id_key(mut self, collapse_id_key: impl Into<String>) -> Self {
        self.collapse_id_key = Some(collapse_id_key.into());
        self
    }

    pub(crate) fn from_payload(payload: CKNotificationInfoPayload) -> Self {
        Self {
            alert_body: payload.alert_body,
            alert_localization_key: payload.alert_localization_key,
            alert_localization_args: payload.alert_localization_args,
            title: payload.title,
            title_localization_key: payload.title_localization_key,
            title_localization_args: payload.title_localization_args,
            subtitle: payload.subtitle,
            subtitle_localization_key: payload.subtitle_localization_key,
            subtitle_localization_args: payload.subtitle_localization_args,
            alert_action_localization_key: payload.alert_action_localization_key,
            alert_launch_image: payload.alert_launch_image,
            sound_name: payload.sound_name,
            desired_keys: payload.desired_keys,
            should_badge: payload.should_badge,
            should_send_content_available: payload.should_send_content_available,
            should_send_mutable_content: payload.should_send_mutable_content,
            category: payload.category,
            collapse_id_key: payload.collapse_id_key,
        }
    }

    pub(crate) fn to_payload(&self) -> CKNotificationInfoPayload {
        CKNotificationInfoPayload {
            alert_body: self.alert_body.clone(),
            alert_localization_key: self.alert_localization_key.clone(),
            alert_localization_args: self.alert_localization_args.clone(),
            title: self.title.clone(),
            title_localization_key: self.title_localization_key.clone(),
            title_localization_args: self.title_localization_args.clone(),
            subtitle: self.subtitle.clone(),
            subtitle_localization_key: self.subtitle_localization_key.clone(),
            subtitle_localization_args: self.subtitle_localization_args.clone(),
            alert_action_localization_key: self.alert_action_localization_key.clone(),
            alert_launch_image: self.alert_launch_image.clone(),
            sound_name: self.sound_name.clone(),
            desired_keys: self.desired_keys.clone(),
            should_badge: self.should_badge,
            should_send_content_available: self.should_send_content_available,
            should_send_mutable_content: self.should_send_mutable_content,
            category: self.category.clone(),
            collapse_id_key: self.collapse_id_key.clone(),
        }
    }
}
