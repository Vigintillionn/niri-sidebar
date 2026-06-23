{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.programs.niri-sidebar;
  tomlFormat = pkgs.formats.toml { };
in
{
  options.programs.niri-sidebar = {
    enable = lib.mkEnableOption "Whether to enable niri-sidebar, a lightweight external sidebar manager for Niri.";

    package = lib.mkOption {
      type = lib.types.nullOr lib.types.package;
      default = null;
      description = "The niri-sidebar package to use.";
    };

    settings = lib.mkOption {
      type = with lib.types; nullOr (oneOf [ tomlFormat.type str path ]);
      default = null;
      description = ''
        Configuration for niri-sidebar. Written to {file}`$XDG_CONFIG_HOME/niri-sidebar/config.toml`.
        Can be a Nix attrset (converted to TOML), a raw TOML string, or a path to a `.toml` file.
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
            focus_peek = 50;
            sticky = false;
          };
          window_rule = [
            {
              app_id = "firefox";
              title = "^Picture-in-Picture$";
              width = 700;
              height = 400;
              focus_peek = 710;
              peek = 10;
              auto_add = true;
            }
          ];
        }
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = lib.optional (cfg.package != null) cfg.package;

    xdg.configFile = lib.mkIf (cfg.settings != null) {
      "niri-sidebar/config.toml".source =
        let
          rawConfig =
            if lib.isString cfg.settings then
              pkgs.writeText "niri-sidebar-config.toml" cfg.settings
            else if builtins.isPath cfg.settings || lib.isStorePath cfg.settings then
              cfg.settings
            else
              tomlFormat.generate "niri-sidebar-config.toml" cfg.settings;
        in
        rawConfig;
    };
  };

  _class = "homeManager";
}
