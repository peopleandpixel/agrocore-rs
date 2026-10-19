/// Centralized role-based permissions for the admin UI.
///
/// This module provides helper functions to check if the current user has
/// permission to view or perform actions on different parts of the application.
///
/// The role hierarchy is:
/// - **SuperAdmin**: Full access to everything, including system-level operations
/// - **Admin**: Full access within their tenant, can manage users, equipment, orders, etc.
/// - **Manager**: Can view and manage most resources, but cannot manage users or system settings
/// - **Worker**: Can view tasks assigned to them, view sites, and perform field operations
/// - **Viewer**: Read-only access to basic information (sites, weather)

use crate::UserRole;

/// Check if the user is a SuperAdmin
pub fn is_super_admin(role: UserRole) -> bool {
    role == UserRole::SuperAdmin
}

/// Check if the user is an Admin or higher
pub fn is_admin_or_higher(role: UserRole) -> bool {
    role == UserRole::SuperAdmin || role == UserRole::Admin
}

/// Check if the user is a Manager or higher (Admin, SuperAdmin)
pub fn is_manager_or_higher(role: UserRole) -> bool {
    role == UserRole::SuperAdmin
        || role == UserRole::Admin
        || role == UserRole::Manager
}

/// Check if the user is a Worker or higher (Manager, Admin, SuperAdmin)
pub fn is_worker_or_higher(role: UserRole) -> bool {
    role != UserRole::Viewer
}

/// Can view the users management page
pub fn can_view_users(role: UserRole) -> bool {
    is_admin_or_higher(role)
}

/// Can manage (create, edit, delete) users
pub fn can_manage_users(role: UserRole) -> bool {
    is_admin_or_higher(role)
}

/// Can view the roles management page
pub fn can_view_roles(role: UserRole) -> bool {
    is_admin_or_higher(role)
}

/// Can manage roles (create, edit, delete)
pub fn can_manage_roles(role: UserRole) -> bool {
    is_admin_or_higher(role)
}

/// Can view the orders/tasks list
pub fn can_view_orders(role: UserRole) -> bool {
    is_worker_or_higher(role)
}

/// Can create new orders/tasks
pub fn can_create_orders(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can edit orders/tasks
pub fn can_edit_orders(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can delete orders/tasks
pub fn can_delete_orders(role: UserRole) -> bool {
    is_admin_or_higher(role)
}

/// Can view the sites management page
pub fn can_view_sites(role: UserRole) -> bool {
    is_worker_or_higher(role)
}

/// Can manage sites (create, edit, delete)
pub fn can_manage_sites(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the equipment management page
pub fn can_view_equipment(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can manage equipment (create, edit, delete)
pub fn can_manage_equipment(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the inventory page
pub fn can_view_inventory(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can manage inventory
pub fn can_manage_inventory(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the finance page
pub fn can_view_finance(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can manage finance
pub fn can_manage_finance(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the analytics page
pub fn can_view_analytics(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the audit log page
pub fn can_view_audit(role: UserRole) -> bool {
    is_admin_or_higher(role)
}

/// Can view the compliance page
pub fn can_view_compliance(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can manage compliance
pub fn can_manage_compliance(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the resources page
pub fn can_view_resources(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can manage resources
pub fn can_manage_resources(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the workers page
pub fn can_view_workers(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can manage workers
pub fn can_manage_workers(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the workforce logs page
pub fn can_view_workforce_logs(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the weather page
pub fn can_view_weather(role: UserRole) -> bool {
    true // Everyone can view weather
}

/// Can manage weather stations
pub fn can_manage_weather(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the settings page
pub fn can_view_settings(role: UserRole) -> bool {
    true // Everyone can view their own settings
}

/// Can manage system settings
pub fn can_manage_settings(role: UserRole) -> bool {
    is_admin_or_higher(role)
}

/// Can view the backups page
pub fn can_view_backups(role: UserRole) -> bool {
    is_admin_or_higher(role)
}

/// Can manage backups
pub fn can_manage_backups(role: UserRole) -> bool {
    is_admin_or_higher(role)
}

/// Can view the import page
pub fn can_view_import(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the map page
pub fn can_view_map(role: UserRole) -> bool {
    is_worker_or_higher(role)
}

/// Can view the groups page
pub fn can_view_groups(role: UserRole) -> bool {
    is_worker_or_higher(role)
}

/// Can manage groups
pub fn can_manage_groups(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the trees page
pub fn can_view_trees(role: UserRole) -> bool {
    is_worker_or_higher(role)
}

/// Can manage trees
pub fn can_manage_trees(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the buildings page
pub fn can_view_buildings(role: UserRole) -> bool {
    is_worker_or_higher(role)
}

/// Can manage buildings
pub fn can_manage_buildings(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the plot entities page
pub fn can_view_plot_entities(role: UserRole) -> bool {
    is_worker_or_higher(role)
}

/// Can manage plot entities
pub fn can_manage_plot_entities(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the livestock page
pub fn can_view_livestock(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can manage livestock
pub fn can_manage_livestock(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the water management page
pub fn can_view_water(role: UserRole) -> bool {
    is_worker_or_higher(role)
}

/// Can manage water
pub fn can_manage_water(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the harvest page
pub fn can_view_harvest(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can manage harvest
pub fn can_manage_harvest(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the agriculture page
pub fn can_view_agriculture(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can manage agriculture
pub fn can_manage_agriculture(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the calculators page
pub fn can_view_calculators(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the customers page
pub fn can_view_customers(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can manage customers
pub fn can_manage_customers(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the applicator licenses page
pub fn can_view_applicator_licenses(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the weather warnings page
pub fn can_view_weather_warnings(role: UserRole) -> bool {
    true // Everyone can view weather warnings
}

/// Can view the sigpac parcels page
pub fn can_view_sigpac(role: UserRole) -> bool {
    is_manager_or_higher(role)
}

/// Can view the wizard page
pub fn can_view_wizard(role: UserRole) -> bool {
    is_worker_or_higher(role)
}

/// Can view the worker tasks page (my tasks)
pub fn can_view_worker_tasks(role: UserRole) -> bool {
    role == UserRole::Worker
}

/// Can impersonate other users
pub fn can_impersonate(role: UserRole) -> bool {
    is_admin_or_higher(role)
}

/// Can activate/deactivate users
pub fn can_toggle_user_status(role: UserRole) -> bool {
    is_admin_or_higher(role)
}
