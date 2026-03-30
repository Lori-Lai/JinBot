# Onshape -> URDF Checklist

## Onshape assembly rules

- Export from a top-level assembly.
- The first instance in the assembly list becomes the base link.
- Every instance in the assembly becomes a link in the export.
- Orphaned links are attached to the base link with a warning.

## Required naming conventions

- `dof_name`: create a joint.
- `dof_name_inv`: create a joint and invert its axis.
- `frame_name`: create a custom frame. In URDF this becomes a dummy link plus a fixed joint.
- `link_name`: rename the exported link for the instance carrying that mate connector.
- `fix_name`: fix two links together so they are merged by the exporter.
- `closing_name`: close a kinematic loop.

## Joint types

- Revolute or cylindrical mate connector -> `revolute`
- Slider mate connector -> `prismatic`
- Fastened mate connector -> `fixed`
- Joint limits set in Onshape are exported unless `ignore_limits` is enabled.

## Joint frame convention

- Joint frames revolve around or translate along the local `z` axis.
- If your axis direction is opposite of what you want in URDF, use the `_inv` suffix.

## URDF-specific notes

- URDF does not support multiple floating base links.
- If the model has multiple base links, use multiple URDFs or add a dummy base link and fix the others to it.
- Gear relations export as `<mimic>`.
- If the robot is fixed to the world, use Onshape's `Fixed` feature.

## Local export flow

1. Fill `.env` with Onshape API credentials.
2. Replace the placeholder URL in `config.json`.
3. Run `.venv/bin/onshape-to-robot .`
4. Inspect `robot.urdf` and meshes in `assets/`

## Optional test tools

- `onshape-to-robot-bullet` needs `pybullet` installed.
- `onshape-to-robot-mujoco` needs MuJoCo-related dependencies installed.
