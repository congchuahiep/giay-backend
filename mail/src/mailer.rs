use askama::Template;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tracing::{error, info};

#[derive(Template)]
#[template(path = "invitation_email.html")]
pub struct InvitationTemplate<'a> {
    pub workspace_name: &'a str,
    pub invite_link: &'a str,
}

#[derive(Clone)]
pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
}

impl Mailer {
    pub fn new(host: &str, user: &str, pass: &str, from_email: &str) -> Self {
        let creds = Credentials::new(user.to_string(), pass.to_string());
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .unwrap()
            .credentials(creds)
            .build();

        Self {
            transport,
            from_email: from_email.to_string(),
        }
    }

    pub async fn send_invitation(
        &self,
        to_email: &str,
        workspace_name: &str,
        token: &str,
    ) -> anyhow::Result<()> {
        let invite_link = format!("http://localhost:3000/invitations/accept?token={}", token);

        let template = InvitationTemplate {
            workspace_name,
            invite_link: &invite_link,
        };

        let html_body = template.render()?;

        let email_msg = Message::builder()
            .from(self.from_email.parse()?)
            .to(to_email.parse()?)
            .subject(format!(
                "You are invited to join the workspace: {}",
                workspace_name
            ))
            .header(ContentType::TEXT_HTML)
            .body(html_body)?;

        match self.transport.send(email_msg).await {
            Ok(_) => {
                info!("Invitation email sent successfully to: {}", to_email);
                Ok(())
            }
            Err(err) => {
                error!("Failed to send email to {}: {:?}", to_email, err);
                Err(anyhow::anyhow!("Failed to send email: {}", err))
            }
        }
    }
}
