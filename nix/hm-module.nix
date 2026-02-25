flake:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.niri-sidebar;
  tomlFormat = pkgs.formats.toml { };
in
{
  options.programs.niri-sidebar = {
    enable = lib.mkEnableOption "niri-sidebar, a sidebar for the Niri window manager";

    package = lib.mkOption {
      type = lib.types.package;
      default = flake.packages.${pkgs.stdenv.hostPlatform.system}.niri-sidebar;
      defaultText = lib.literalExpression "flake.packages.\${pkgs.stdenv.hostPlatform.system}.niri-sidebar";
      description = "The niri-sidebar package to use.";
    };

    settings = lib.mkOption {
      type = tomlFormat.type;
      default = { };
      description = ''
        Configuration for niri-sidebar, serialized to config.toml.
        See https://github.com/Vigintillionn/niri-sidebar for options.
      '';
      example = lib.literalExpression ''
        {
          geometry = {
            width = 400;
            height = 335;
            gap = 10;
          };
          margins = {
            top = 50;
            right = 10;
            left = 10;
            bottom = 10;
          };
          interaction = {
            position = "right";
            peek = 10;
            sticky = false;
          };
          window_rule = [
            {
              app_id = "firefox";
              title = "^Picture-in-Picture$";
              width = 700;
              height = 400;
              auto_add = true;
            }
          ];
        }
      '';
    };

    systemd.enable = lib.mkEnableOption "the niri-sidebar listen daemon as a systemd user service";
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile."niri-sidebar/config.toml" = lib.mkIf (cfg.settings != { }) {
      source = tomlFormat.generate "niri-sidebar-config" cfg.settings;
    };

    systemd.user.services.niri-sidebar-listen = lib.mkIf cfg.systemd.enable {
      Unit = {
        Description = "niri-sidebar listen daemon";
        PartOf = [ "graphical-session.target" ];
        After = [ "graphical-session.target" ];
      };
      Service = {
        ExecStart = "${lib.getExe cfg.package} listen";
        Restart = "on-failure";
        RestartSec = 5;
      };
      Install = {
        WantedBy = [ "graphical-session.target" ];
      };
    };
  };
}
