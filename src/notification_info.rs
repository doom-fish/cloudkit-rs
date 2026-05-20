use crate::private::CKNotificationInfoPayload;

/// Wraps `CKNotificationInfo`.
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
    /// Creates a wrapper mirroring `CKNotificationInfo`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mirrors `CKNotificationInfo.alertBody`.
    pub fn alert_body(&self) -> Option<&str> {
        self.alert_body.as_deref()
    }

    /// Mirrors `CKNotificationInfo.alertLocalizationKey`.
    pub fn alert_localization_key(&self) -> Option<&str> {
        self.alert_localization_key.as_deref()
    }

    /// Mirrors `CKNotificationInfo.alertLocalizationArgs`.
    pub fn alert_localization_args(&self) -> Option<&[String]> {
        self.alert_localization_args.as_deref()
    }

    /// Mirrors `CKNotificationInfo.title`.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Mirrors `CKNotificationInfo.titleLocalizationKey`.
    pub fn title_localization_key(&self) -> Option<&str> {
        self.title_localization_key.as_deref()
    }

    /// Mirrors `CKNotificationInfo.titleLocalizationArgs`.
    pub fn title_localization_args(&self) -> Option<&[String]> {
        self.title_localization_args.as_deref()
    }

    /// Mirrors `CKNotificationInfo.subtitle`.
    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    /// Mirrors `CKNotificationInfo.subtitleLocalizationKey`.
    pub fn subtitle_localization_key(&self) -> Option<&str> {
        self.subtitle_localization_key.as_deref()
    }

    /// Mirrors `CKNotificationInfo.subtitleLocalizationArgs`.
    pub fn subtitle_localization_args(&self) -> Option<&[String]> {
        self.subtitle_localization_args.as_deref()
    }

    /// Mirrors `CKNotificationInfo.alertActionLocalizationKey`.
    pub fn alert_action_localization_key(&self) -> Option<&str> {
        self.alert_action_localization_key.as_deref()
    }

    /// Mirrors `CKNotificationInfo.alertLaunchImage`.
    pub fn alert_launch_image(&self) -> Option<&str> {
        self.alert_launch_image.as_deref()
    }

    /// Mirrors `CKNotificationInfo.soundName`.
    pub fn sound_name(&self) -> Option<&str> {
        self.sound_name.as_deref()
    }

    /// Mirrors `CKNotificationInfo.desiredKeys`.
    pub fn desired_keys(&self) -> Option<&[String]> {
        self.desired_keys.as_deref()
    }

    /// Mirrors `CKNotificationInfo.shouldBadge`.
    pub const fn should_badge(&self) -> bool {
        self.should_badge
    }

    /// Mirrors `CKNotificationInfo.shouldSendContentAvailable`.
    pub const fn should_send_content_available(&self) -> bool {
        self.should_send_content_available
    }

    /// Mirrors `CKNotificationInfo.shouldSendMutableContent`.
    pub const fn should_send_mutable_content(&self) -> bool {
        self.should_send_mutable_content
    }

    /// Mirrors `CKNotificationInfo.category`.
    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    /// Mirrors `CKNotificationInfo.collapseIDKey`.
    pub fn collapse_id_key(&self) -> Option<&str> {
        self.collapse_id_key.as_deref()
    }

    /// Sets the value mirroring `CKNotificationInfo.alertBody`.
    pub fn with_alert_body(mut self, alert_body: impl Into<String>) -> Self {
        self.alert_body = Some(alert_body.into());
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.alertLocalizationKey`.
    pub fn with_alert_localization_key(mut self, key: impl Into<String>) -> Self {
        self.alert_localization_key = Some(key.into());
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.alertLocalizationArgs`.
    pub fn with_alert_localization_args(mut self, args: Vec<String>) -> Self {
        self.alert_localization_args = Some(args);
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.title`.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.titleLocalizationKey`.
    pub fn with_title_localization_key(mut self, key: impl Into<String>) -> Self {
        self.title_localization_key = Some(key.into());
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.titleLocalizationArgs`.
    pub fn with_title_localization_args(mut self, args: Vec<String>) -> Self {
        self.title_localization_args = Some(args);
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.subtitle`.
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.subtitleLocalizationKey`.
    pub fn with_subtitle_localization_key(mut self, key: impl Into<String>) -> Self {
        self.subtitle_localization_key = Some(key.into());
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.subtitleLocalizationArgs`.
    pub fn with_subtitle_localization_args(mut self, args: Vec<String>) -> Self {
        self.subtitle_localization_args = Some(args);
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.alertActionLocalizationKey`.
    pub fn with_alert_action_localization_key(mut self, key: impl Into<String>) -> Self {
        self.alert_action_localization_key = Some(key.into());
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.alertLaunchImage`.
    pub fn with_alert_launch_image(mut self, image: impl Into<String>) -> Self {
        self.alert_launch_image = Some(image.into());
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.soundName`.
    pub fn with_sound_name(mut self, sound_name: impl Into<String>) -> Self {
        self.sound_name = Some(sound_name.into());
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.desiredKeys`.
    pub fn with_desired_keys(mut self, desired_keys: Vec<String>) -> Self {
        self.desired_keys = Some(desired_keys);
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.shouldBadge`.
    pub fn with_should_badge(mut self, should_badge: bool) -> Self {
        self.should_badge = should_badge;
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.contentAvailable`.
    pub fn with_content_available(mut self, should_send_content_available: bool) -> Self {
        self.should_send_content_available = should_send_content_available;
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.mutableContent`.
    pub fn with_mutable_content(mut self, should_send_mutable_content: bool) -> Self {
        self.should_send_mutable_content = should_send_mutable_content;
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.category`.
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Sets the value mirroring `CKNotificationInfo.collapseIDKey`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_notification_info_matches_expected_flags() {
        let info = CKNotificationInfo::new();

        assert!(info.alert_body().is_none());
        assert!(info.alert_localization_key().is_none());
        assert!(info.alert_localization_args().is_none());
        assert!(info.title().is_none());
        assert!(info.desired_keys().is_none());
        assert!(!info.should_badge());
        assert!(info.should_send_content_available());
        assert!(!info.should_send_mutable_content());
        assert!(info.category().is_none());
        assert!(info.collapse_id_key().is_none());
    }

    #[test]
    fn alert_builders_populate_expected_accessors() {
        let args = vec!["first".to_owned(), "second".to_owned()];
        let info = CKNotificationInfo::new()
            .with_alert_body("body")
            .with_alert_localization_key("LOC_KEY")
            .with_alert_localization_args(args.clone())
            .with_alert_action_localization_key("ACTION_KEY")
            .with_alert_launch_image("launch.png");

        assert_eq!(info.alert_body(), Some("body"));
        assert_eq!(info.alert_localization_key(), Some("LOC_KEY"));
        assert_eq!(info.alert_localization_args(), Some(args.as_slice()));
        assert_eq!(info.alert_action_localization_key(), Some("ACTION_KEY"));
        assert_eq!(info.alert_launch_image(), Some("launch.png"));
    }

    #[test]
    fn title_and_delivery_builders_populate_expected_accessors() {
        let title_args = vec!["title".to_owned()];
        let subtitle_args = vec!["subtitle".to_owned(), "detail".to_owned()];
        let desired_keys = vec!["name".to_owned(), "updatedAt".to_owned()];
        let info = CKNotificationInfo::new()
            .with_title("Hello")
            .with_title_localization_key("TITLE_KEY")
            .with_title_localization_args(title_args.clone())
            .with_subtitle("World")
            .with_subtitle_localization_key("SUBTITLE_KEY")
            .with_subtitle_localization_args(subtitle_args.clone())
            .with_sound_name("ping.aiff")
            .with_desired_keys(desired_keys.clone())
            .with_should_badge(true)
            .with_content_available(false)
            .with_mutable_content(true)
            .with_category("updates")
            .with_collapse_id_key("thread-1");

        assert_eq!(info.title(), Some("Hello"));
        assert_eq!(info.title_localization_key(), Some("TITLE_KEY"));
        assert_eq!(info.title_localization_args(), Some(title_args.as_slice()));
        assert_eq!(info.subtitle(), Some("World"));
        assert_eq!(info.subtitle_localization_key(), Some("SUBTITLE_KEY"));
        assert_eq!(info.subtitle_localization_args(), Some(subtitle_args.as_slice()));
        assert_eq!(info.sound_name(), Some("ping.aiff"));
        assert_eq!(info.desired_keys(), Some(desired_keys.as_slice()));
        assert!(info.should_badge());
        assert!(!info.should_send_content_available());
        assert!(info.should_send_mutable_content());
        assert_eq!(info.category(), Some("updates"));
        assert_eq!(info.collapse_id_key(), Some("thread-1"));
    }

    #[test]
    fn payload_round_trip_preserves_notification_info() {
        let info = CKNotificationInfo::new()
            .with_alert_body("body")
            .with_title("title")
            .with_subtitle("subtitle")
            .with_sound_name("ding.aiff")
            .with_desired_keys(vec!["name".into(), "owner".into()])
            .with_should_badge(true)
            .with_mutable_content(true)
            .with_category("updates")
            .with_collapse_id_key("thread-7");

        assert_eq!(CKNotificationInfo::from_payload(info.to_payload()), info);
    }
}
