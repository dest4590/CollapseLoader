import Home from "../views/Home.vue";
import News from "../views/News.vue";
import Settings from "../views/Settings.vue";
import About from "../views/About.vue";
import Customization from "../views/Customization.vue";
import CustomClients from "../views/CustomClients.vue";
import AppLogs from "../views/AppLogs.vue";
import AccountView from "../views/AccountView.vue";
import LoginView from "../views/LoginView.vue";
import RegisterView from "../views/RegisterView.vue";
import VerifyEmailView from "../views/VerifyEmailView.vue";
import FriendsView from "../views/FriendsView.vue";
import UserProfileView from "../views/UserProfileView.vue";
import Marketplace from "../views/Marketplace.vue";
import NetworkDebug from "../views/NetworkDebug.vue";

export const views: Record<string, any> = {
    home: Home,
    news: News,
    settings: Settings,
    about: About,
    customization: Customization,
    custom_clients: CustomClients,
    app_logs: AppLogs,
    account: AccountView,
    login: LoginView,
    register: RegisterView,
    verify: VerifyEmailView,
    friends: FriendsView,
    "user-profile": UserProfileView,
    marketplace: Marketplace,
    network_debug: NetworkDebug,
};

export const tabOrder = [
    "home",
    "custom_clients",
    "friends",
    "settings",
    "customization",
    "app_logs",
    "account",
    "login",
    "register",
    "about",
];
