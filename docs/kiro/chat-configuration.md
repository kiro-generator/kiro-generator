URL Source: https://kiro.dev/docs/cli/chat/configuration/
Scraped: 2026-02-19T21:00:26Z

---

Title: Configuration - CLI - Docs - Kiro

URL Source: https://kiro.dev/docs/cli/chat/configuration/

Markdown Content:
Configuration - CLI - Docs - Kiro
===============

Select your cookie preferences
------------------------------

We use essential cookies and similar tools that are necessary to provide our site and services. We use performance cookies to collect anonymous statistics, so we can understand how customers use our site and make improvements. Essential cookies cannot be deactivated, but you can choose “Customize” or “Decline” to decline performance cookies. 

 If you agree, AWS and approved third parties will also use cookies to provide useful site features, remember your preferences, and display relevant content, including relevant advertising. To accept or decline all non-essential cookies, choose “Accept” or “Decline.” To make more detailed choices, choose “Customize.”

Accept Decline Customize

Customize cookie preferences
----------------------------

We use cookies and similar tools (collectively, "cookies") for the following purposes.

### Essential

Essential cookies are necessary to provide our site and services and cannot be deactivated. They are usually set in response to your actions on the site, such as setting your privacy preferences, signing in, or filling in forms.

### Performance

Performance cookies provide anonymous statistics about how customers navigate our site so we can improve site experience and performance. Approved third parties may perform analytics on our behalf, but they cannot use the data for their own purposes.

- [x]  

Allowed

### Functional

Functional cookies help us provide useful site features, remember your preferences, and display relevant content. Approved third parties may set these cookies to provide certain site features. If you do not allow these cookies, then some or all of these services may not function properly.

- [x]  

Allowed

### Advertising

Advertising cookies may be set through our site by us or our advertising partners and help us deliver relevant marketing content. If you do not allow these cookies, you will experience less relevant advertising.

- [x]  

Allowed

Blocking some types of cookies may impact your experience of our sites. You may review and change your choices at any time by selecting Cookie preferences in the footer of this site. We and selected third-parties use cookies or similar technologies as specified in the[AWS Cookie Notice](https://aws.amazon.com/legal/cookies/).

Cancel Save preferences

Your privacy choices
--------------------

We and our advertising partners (“we”) may use information we collect from or about you to show you ads on other websites and online services. Under certain laws, this activity is referred to as “cross-context behavioral advertising” or “targeted advertising.”

To opt out of our use of cookies or similar technologies to engage in these activities, select “Opt out of cross-context behavioral ads” and “Save preferences” below. If you clear your browser cookies or visit this site from a different device or browser, you will need to make your selection again. For more information about cookies and how we use them, read our[Cookie Notice](https://aws.amazon.com/legal/cookies/).

Allow cross-context behavioral ads Opt out of cross-context behavioral ads 

 

To opt out of the use of other identifiers, such as contact information, for these activities, fill out the form[here](https://pulse.aws/application/ZRPLWLL6?p=0).

For more information about how AWS handles your information, read the[AWS Privacy Notice](https://aws.amazon.com/privacy/).

Cancel Save preferences

Unable to save cookie preferences
---------------------------------

We will only store essential cookies at this time, because we were unable to save your cookie preferences.

If you want to change your cookie preferences, try again later using the link in the AWS console footer, or contact support if the problem persists.

Dismiss

[![Image 1: Kiro](https://kiro.dev/images/kiro-wordmark.png?h=0ad65a93)](https://kiro.dev/)

*   [CLI](https://kiro.dev/cli/)
*   [Powers](https://kiro.dev/powers/)
*   [Autonomous agent](https://kiro.dev/autonomous-agent/)
*   [Enterprise](https://kiro.dev/enterprise/)
*   [Pricing](https://kiro.dev/pricing/)
*   [Docs](https://kiro.dev/docs/)
*   Resources

K

[SIGN IN](https://app.kiro.dev/)[DOWNLOADS](https://kiro.dev/downloads/)

[![Image 2: Kiro](https://kiro.dev/images/kiro-wordmark.png?h=0ad65a93)](https://kiro.dev/)

K

IDE CLI Autonomous

[Get started](https://kiro.dev/docs/cli/)

[Chat](https://kiro.dev/docs/cli/chat/)

[Model selection](https://kiro.dev/docs/cli/chat/model-selection/)

[Session management](https://kiro.dev/docs/cli/chat/session-management/)

[Subagents](https://kiro.dev/docs/cli/chat/subagents/)

[Planning Agent](https://kiro.dev/docs/cli/chat/planning-agent/)

[Help Agent](https://kiro.dev/docs/cli/chat/help-agent/)

[Prompts](https://kiro.dev/docs/cli/chat/manage-prompts/)

[File references](https://kiro.dev/docs/cli/chat/file-references/)

[Context management](https://kiro.dev/docs/cli/chat/context/)

[Responding to messages](https://kiro.dev/docs/cli/chat/responding/)

[Permissions](https://kiro.dev/docs/cli/chat/permissions/)

[Working with Git](https://kiro.dev/docs/cli/chat/git-aware-selection/)

[Images](https://kiro.dev/docs/cli/chat/images/)

[Security considerations](https://kiro.dev/docs/cli/chat/security/)

[Configuration](https://kiro.dev/docs/cli/chat/configuration/)

[Custom diff tools](https://kiro.dev/docs/cli/chat/diff-tools/)

[Custom agents](https://kiro.dev/docs/cli/custom-agents/)

[MCP](https://kiro.dev/docs/cli/mcp/)

[Steering](https://kiro.dev/docs/cli/steering/)[Agent Skills](https://kiro.dev/docs/cli/skills/)[ACP](https://kiro.dev/docs/cli/acp/)

[Experimental](https://kiro.dev/docs/cli/experimental/)

[Hooks](https://kiro.dev/docs/cli/hooks/)[Auto complete](https://kiro.dev/docs/cli/autocomplete/)[Code Intelligence](https://kiro.dev/docs/cli/code-intelligence/)

[Billing for individuals](https://kiro.dev/docs/cli/billing/)

[Enterprise billing](https://kiro.dev/docs/cli/enterprise/getting-started/)

[Privacy and security](https://kiro.dev/docs/cli/privacy-and-security/)

[Reference](https://kiro.dev/docs/cli/reference/cli-commands/)

[Upgrading from Q CLI](https://kiro.dev/docs/cli/migrating-from-q/)

1.   [Docs](https://kiro.dev/docs)

3.   [CLI](https://kiro.dev/docs/cli)

5.   [Chat](https://kiro.dev/docs/cli/chat)

7.   Configuration

On this page

Configuration
=============

### Configuration file paths[](https://kiro.dev/docs/cli/chat/configuration/#configuration-file-paths)

You can configure Kiro CLI to match your development preferences and team standards. You can set configuration in one of three scopes:

1.   **Global** - Configuration that is applied across all the projects where Kiro is used - `<user-home>/.kiro/`
2.   **Project** - Configuration specific to a project - `<project-root>/.kiro`
3.   **Agent** - Configuration defined in the agent configuration file - `<user-home | project-root>/.kiro/agents`

| Configuration | Global Scope | Project Scope |
| --- | --- | --- |
| MCP servers | `~/.kiro/settings/mcp.json` | `.kiro/settings/mcp.json` |
| Prompts | `~/.kiro/prompts` | `.kiro/prompts` |
| Custom agents | `~/.kiro/agents` | `.kiro/agents` |
| Steering | `~/.kiro/steering` | `.kiro/steering` |
| Settings | `~/.kiro/settings/cli.json` |  |

### What can you configure at these scopes[](https://kiro.dev/docs/cli/chat/configuration/#what-can-you-configure-at-these-scopes)

| Configuration | User Scope | Project Scope | Agent Scope |
| --- | --- | --- | --- |
| MCP servers | Yes | Yes | Yes |
| Prompts | Yes | Yes | No |
| Custom agents | Yes | Yes | N/A |
| Steering | Yes | Yes | Yes |
| Settings | Yes | N/A | N/A |

### Resolving configuration conflicts[](https://kiro.dev/docs/cli/chat/configuration/#resolving-configuration-conflicts)

Configuration conflicts are resolved by selecting the configuration that is closest to where you are interacting with Kiro CLI. For example, if you have a MCP configuration in both global and project `mcp.json` files, when you are chatting with Kiro in the project folder, the MCP configuration from the project folder will be applied.

Since you can also define a custom agents at a global and project scope, if there is a conflict between at the same level with the agent configuration, then Kiro CLI will choose the configuration from the agent.

Here's the priority order of how configuration is rationalized:

| Configuration | Priority |
| --- | --- |
| MCP servers | Agent > Project > Global |
| Prompts | Project > Global |
| Custom agents | Project > Global |
| Steering | Project > Global |

Since MCP servers can be configured in three scopes and there is a `includeMcpJson` setting in an agent configuration, MCP servers are handled slightly differently. Refer [MCP server loading priority](https://kiro.dev/docs/cli/mcp/#mcp-server-loading-priority)

Page updated: December 10, 2025

[Security considerations](https://kiro.dev/docs/cli/chat/security/)

[Custom diff tools](https://kiro.dev/docs/cli/chat/diff-tools/)

On this page

*   [Configuration file paths](https://kiro.dev/docs/cli/chat/configuration/#configuration-file-paths)
*   [What can you configure at these scopes](https://kiro.dev/docs/cli/chat/configuration/#what-can-you-configure-at-these-scopes)
*   [Resolving configuration conflicts](https://kiro.dev/docs/cli/chat/configuration/#resolving-configuration-conflicts)

* * *

![Image 3: Kiro](https://kiro.dev/images/kiro-wordmark.png?h=0ad65a93)

Product

*   [About Kiro](https://kiro.dev/about/)
*   [CLI](https://kiro.dev/cli/)
*   [Powers](https://kiro.dev/powers/)
*   [Autonomous agent](https://kiro.dev/autonomous-agent/)
*   [Pricing](https://kiro.dev/pricing/)
*   [Downloads](https://kiro.dev/downloads/)

For

*   [Enterprise](https://kiro.dev/enterprise/)
*   [Startups](https://kiro.dev/startups/)

Resources

*   [Documentation](https://kiro.dev/docs/)
*   [Blog](https://kiro.dev/blog/)
*   [Changelog](https://kiro.dev/changelog/)
*   [FAQs](https://kiro.dev/faq/)
*   [Report a bug](https://github.com/kirodotdev/Kiro/issues/new/choose)
*   [Suggest an idea](https://github.com/kirodotdev/Kiro/issues/new?template=feature_request.yml)
*   [Billing support](https://support.aws.amazon.com/#/contacts/kiro)

Social

*   [](https://discord.gg/kirodotdev)
*   [](https://www.linkedin.com/showcase/kirodotdev)
*   [](https://x.com/kirodotdev)
*   [](https://www.instagram.com/kirodotdev)
*   [](https://www.youtube.com/@kirodotdev)
*   [](https://bsky.app/profile/kiro.dev)
*   [](https://www.twitch.tv/kirodotdev)

[](https://aws.amazon.com/)

[Site Terms](https://aws.amazon.com/terms/)[License](https://kiro.dev/license/)[Responsible AI Policy](https://aws.amazon.com/ai/responsible-ai/policy/)[Legal](https://aws.amazon.com/legal/)[Privacy Policy](https://aws.amazon.com/privacy/)[Cookie Preferences](https://kiro.dev/docs/cli/chat/configuration/#)
