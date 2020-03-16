use crate::rules::error::RuleError;
use crate::rules::functions::*;
use crate::rules::RuleContext;
use dino_park_trust::GroupsTrust;
use log::info;

pub const CREATE_GROUP: Engine = Engine {
    rules: &[&rule_is_creator, &rule_valid_group_name],
};

pub const CURRENT_USER_CAN_JOIN: Engine = Engine {
    rules: &[&current_user_can_join],
};

pub const CURRENT_USER_CAN_REQUEST: Engine = Engine {
    rules: &[&current_user_can_join, &is_reviewed_group],
};

pub const SEARCH_USERS: Engine = Engine {
    rules: &[&rule_host_can_invite],
};

pub const DELETE_INVITATION: Engine = Engine {
    rules: &[&rule_host_can_invite],
};

pub const INVITE_MEMBER: Engine = Engine {
    rules: &[&rule_host_can_invite, &member_can_join, &user_not_a_member],
};

pub const RENEW_MEMBER: Engine = Engine {
    rules: &[&rule_host_can_invite, &rule_user_has_member_role],
};

pub const REMOVE_MEMBER: Engine = Engine {
    rules: &[&rule_host_can_remove],
};

pub const EDIT_TERMS: Engine = Engine {
    rules: &[&rule_host_can_edit_terms],
};

pub const CAN_ADD_CURATOR: Engine = Engine {
    rules: &[&rule_host_is_curator, &member_is_ndaed],
};

pub const HOST_IS_CURATOR: Engine = Engine {
    rules: &[&rule_host_is_curator],
};

pub const HOST_IS_GROUP_ADMIN: Engine = Engine {
    rules: &[&rule_host_is_group_admin],
};

pub const ONLY_ADMINS: Engine = Engine {
    rules: &[&rule_only_admins],
};

pub struct Engine<'a> {
    pub rules: &'a [&'static Rule],
}

impl<'a> Engine<'a> {
    pub fn run(self: &Self, ctx: &RuleContext) -> Result<(), RuleError> {
        let ok = self.rules.iter().try_for_each(|rule| rule(ctx));
        if ok.is_err() && ctx.scope_and_user.groups_scope == GroupsTrust::Admin {
            info!("using admin privileges for {}", ctx.host_uuid);
            return Ok(());
        }
        ok
    }
}
