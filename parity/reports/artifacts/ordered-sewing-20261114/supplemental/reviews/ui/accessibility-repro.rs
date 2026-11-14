// Insert inside existing tests::ordered_sewing; reuses that module's fixture and harness helpers.
        #[test]
        fn review_new_group_exposes_a_boundary_add_action_without_canvas_pointer_input() {
            let ctx=egui::Context::default();ctx.enable_accesskit();
            let (mut app,curves)=fixture(false);
            let piece=app.editor.project.pieces[0].id;
            app.select_curve(Some((piece,curves[0])));
            let output=frame(&ctx,&mut app,vec![]);
            let output=click(&ctx,&mut app,&output,"M:N sewing");
            assert!(!buttons(&output,"Add boundary to A").is_empty(),"M:N creation offers no action to append the inspector-selected boundary without a canvas pointer hit");
            stop_jobs(&mut app);
        }
