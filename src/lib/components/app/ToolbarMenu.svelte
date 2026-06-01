<script lang="ts">
    import { toast } from "svelte-sonner";

    import { redo, undo } from "$lib/actions.js";
    import { apiLoadProject, apiSaveProject, buildProblemConfig } from "$lib/api.js";
    import { GridMode, ZoomLevel, app, resetApp, view } from "$lib/app.svelte.js";
    import * as Menubar from "$lib/components/ui/menubar/index.js";
    import type { CheckpointRecord, ProjectFile } from "$lib/schemas.js";

    async function onSave() {
        const config = buildProblemConfig({
            startDate: app.startDate,
            people: app.people,
            weekdayHours: app.weekdayHours,
            bankHolidayDefaultHours: app.bankHolidayDefaultHours,
            bankHolidays: app.bankHolidays,
            customOverrides: app.customOverrides,
            skipLastShifts: app.skipLastShifts,
        });

        const project: ProjectFile = {
            version: 1,
            config,
            raw: {
                bank_holidays: app.bankHolidays.map((bh) => ({
                    date: bh.date,
                    name: bh.name,
                    enabled: bh.enabled,
                    lead_hours: bh.lead_hours,
                    support_hours: bh.support_hours,
                })),
                bank_holiday_default_hours: app.bankHolidayDefaultHours,
                custom_overrides: app.customOverrides.map((co) => ({
                    date: co.date,
                    role: co.role,
                    hours: co.hours,
                })),
            },
            solution: [...app.slots],
            solver: app.solverParams,
            refiner: app.refinerParams,
            weights: app.weights,
            top_k: app.topK,
            checkpoints: app.checkpoints.map(
                (cp): CheckpointRecord => ({
                    name: cp.name,
                    slots: [...cp.slots],
                    timestamp: cp.timestamp,
                }),
            ),
        };

        try {
            await apiSaveProject(project);
            toast.success("Project saved");
        } catch (e) {
            toast.error("Failed to save project", { description: String(e) });
        }
    }

    async function onLoad() {
        try {
            const project = await apiLoadProject();
            if (!project) return;

            // Hydrate config
            app.startDate = project.config.start_date;
            app.people = project.config.people.map((p) => ({ ...p }));
            app.weekdayHours = project.config.weekday_hours.map(
                ([a, b]) => [a, b] as [number, number],
            );
            app.skipLastShifts = project.config.skip_last_shifts;

            // Hydrate raw settings
            app.bankHolidays = project.raw.bank_holidays.map((bh) => ({
                date: bh.date,
                name: bh.name,
                enabled: bh.enabled,
                lead_hours: bh.lead_hours,
                support_hours: bh.support_hours,
            }));
            app.bankHolidayDefaultHours = project.raw.bank_holiday_default_hours.map(
                ([a, b]) => [a, b] as [number, number],
            );
            app.customOverrides = project.raw.custom_overrides.map((co) => ({
                date: co.date,
                role: co.role as "Lead" | "Support",
                hours: co.hours,
            }));

            // Hydrate solution
            app.loadSlots(project.solution);

            // Hydrate parameters
            app.solverParams = project.solver;
            app.refinerParams = project.refiner;
            app.weights = project.weights;
            app.topK = project.top_k;

            // Hydrate checkpoints
            app.checkpoints = project.checkpoints.map((cp) => ({
                name: cp.name,
                slots: [...cp.slots],
                timestamp: cp.timestamp,
            }));

            toast.success("Project loaded");
        } catch (e) {
            toast.error("Failed to load project", { description: String(e) });
        }
    }
</script>

<Menubar.Root>
    <Menubar.Menu>
        <Menubar.Trigger>File</Menubar.Trigger>

        <Menubar.Content>
            <Menubar.Item onclick={resetApp}>New</Menubar.Item>
            <Menubar.Item onclick={onLoad}>Open</Menubar.Item>
            <Menubar.Item onclick={onSave}>Save</Menubar.Item>
            <Menubar.Item>Export PDF</Menubar.Item>
        </Menubar.Content>
    </Menubar.Menu>

    <Menubar.Menu>
        <Menubar.Trigger>Edit</Menubar.Trigger>

        <Menubar.Content>
            <Menubar.Item onclick={undo} disabled={!app.history.canUndo}>
                Undo <Menubar.Shortcut>⌘Z</Menubar.Shortcut>
            </Menubar.Item>
            <Menubar.Item onclick={redo} disabled={!app.history.canRedo}>
                Redo <Menubar.Shortcut>⌘Y</Menubar.Shortcut>
            </Menubar.Item>
        </Menubar.Content>
    </Menubar.Menu>

    <Menubar.Menu>
        <Menubar.Trigger>View</Menubar.Trigger>

        <Menubar.Content>
            <Menubar.RadioGroup bind:value={view.zoom}>
                <Menubar.GroupHeading>Zoom level</Menubar.GroupHeading>
                <Menubar.RadioItem value={ZoomLevel.Micro}>Micro</Menubar.RadioItem>
                <Menubar.RadioItem value={ZoomLevel.Standard}>Standard</Menubar.RadioItem>
                <Menubar.RadioItem value={ZoomLevel.Detail}>Detail</Menubar.RadioItem>
            </Menubar.RadioGroup>

            <Menubar.Separator />

            <Menubar.RadioGroup bind:value={view.gridMode}>
                <Menubar.GroupHeading>Grid mode</Menubar.GroupHeading>
                <Menubar.RadioItem value={GridMode.Bars}>Bars</Menubar.RadioItem>
                <Menubar.RadioItem value={GridMode.Filled}>Filled</Menubar.RadioItem>
            </Menubar.RadioGroup>

            <Menubar.Separator />

            <Menubar.CheckboxItem bind:checked={view.showConflicts}>
                Show conflicts
            </Menubar.CheckboxItem>

            <Menubar.CheckboxItem bind:checked={view.showHolidays}>
                Show holidays
            </Menubar.CheckboxItem>
        </Menubar.Content>
    </Menubar.Menu>
</Menubar.Root>
