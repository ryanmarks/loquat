use std::ops::Deref;

use loquat_common::api::a1_2010_report::UpdateBody;

use yew::prelude::*;
use yewdux::prelude::use_store;

use loquat_common::models::{
    A1Standard2010Determination, A1Standard2010Report, FanSeries, FanSize,
};

use crate::api::store::Store as ApiStore;
use crate::common::components::determination_table::TaggedInput;
use crate::common::components::{DeterminationsPasteTextArea, FanSeriesAndSizePicker};
use crate::features::a1_2010_report::components::{A12010DeterminationTable, A1FanPlot};
use crate::store::select_a1_report;
use crate::{
    api::store::{ApiRequestAction, GetParameters, Gettable},
    store::use_app_store_selector_with_deps,
};

#[derive(Properties, PartialEq)]
pub struct EditA1PageProps {
    pub id: Option<String>,
}

#[function_component]
pub fn EditA1Page(props: &EditA1PageProps) -> Html {
    let (_state, api_dispatch) = use_store::<ApiStore>();

    let report_id = props.id.as_ref().map(|id| id.replace("%20", " "));
    let report_id_state: UseStateHandle<String> = {
        let report_id = report_id.clone();
        use_state(move || report_id.map_or("".to_string(), |id| id.to_string()))
    };
    let picked_fan_series_state: UseStateHandle<Option<FanSeries<()>>> = use_state(|| None);
    let picked_fan_size_state: UseStateHandle<Option<FanSize<()>>> = use_state(|| None);
    let entered_rpm_state: UseStateHandle<String> = use_state(|| "".to_string());
    let determinations_state: UseStateHandle<Vec<[String; 3]>> = use_state(|| vec![]);

    type FanSizeIdParse = Result<String, Vec<String>>;
    type RpmParse = Result<f64, Vec<String>>;

    type DeterminationValueErr = Vec<String>;
    type DeterminationRowErr = [DeterminationValueErr; 3];
    type DeterminationsParseErr = Vec<DeterminationRowErr>;
    type DeterminationsParse = Result<Vec<A1Standard2010Determination>, DeterminationsParseErr>;

    #[derive(Debug, Clone, PartialEq)]
    struct UpdateBodyErrors {
        size_errs: Vec<String>,
        rpm_errs: Vec<String>,
        determination_errs: DeterminationsParseErr,
    }

    let parsed_update_body: Result<UpdateBody, UpdateBodyErrors> = (|| {
        let parsed_fan_size_id: FanSizeIdParse = match picked_fan_size_state.deref() {
            Some(fan_size) => Ok(fan_size.id.clone()),
            None => Err(vec!["You must select a fan size for the report".to_string()]),
        };

        let parsed_rpm: RpmParse = entered_rpm_state
            .deref()
            .parse::<f64>()
            .map_err(|_| {
                if entered_rpm_state.is_empty() {
                    vec!["You must enter a fan RPM for the test".to_string()]
                } else {
                    vec!["You must enter a valid number".to_string()]
                }
            })
            .and_then(|value| {
                if value <= 0.0 {
                    return Err(vec!["The fan speed must be positive".to_string()]);
                }
                Ok(value)
            });

        let parsed_determinations: DeterminationsParse = {
            let parsed_rows: Vec<Result<[f64; 3], [DeterminationValueErr; 3]>> =
                determinations_state
                    .deref()
                    .iter()
                    .map(|det| {
                        let det_attempt: [Result<f64, Vec<String>>; 3] = det.clone().map(|x| {
                            x.parse::<f64>().map_err(|_| {
                                if x.is_empty() {
                                    vec!["All determination point values must be entered"
                                        .to_string()]
                                } else {
                                    vec!["You must enter a valid number".to_string()]
                                }
                            })
                        });
                        if det_attempt.iter().all(|d| d.is_ok()) {
                            Ok(det_attempt.map(|d| d.ok().unwrap()))
                        } else {
                            Err(det_attempt.map(|a| a.err().unwrap_or_default()))
                        }
                    })
                    .collect::<Vec<_>>();

            if parsed_rows.iter().all(|d| d.is_ok()) {
                Ok(parsed_rows
                    .into_iter()
                    .map(|d| {
                        let [static_pressure, cfm, brake_horsepower] = d.ok().unwrap();
                        A1Standard2010Determination {
                            static_pressure,
                            cfm,
                            brake_horsepower,
                        }
                    })
                    .collect())
            } else {
                Err(parsed_rows
                    .into_iter()
                    .map(|row| row.err().unwrap_or_default())
                    .collect::<Vec<_>>())
            }
        };

        let parses = (parsed_fan_size_id, parsed_rpm, parsed_determinations);
        if let (Ok(fan_size_id), Ok(fan_rpm), Ok(determinations)) = parses {
            let id: String = report_id_state.to_string();
            Ok(UpdateBody {
                id,
                determinations,
                fan_rpm,
                fan_size_id,
            })
        } else {
            Err(UpdateBodyErrors {
                size_errs: parses.0.err().unwrap_or_default(),
                rpm_errs: parses.1.err().unwrap_or_default(),
                determination_errs: parses.2.err().unwrap_or_default(),
            })
        }
    })();

    let maybe_report: Option<A1Standard2010Report<FanSize<FanSeries<()>>>> =
        use_app_store_selector_with_deps(select_a1_report, report_id.clone());
    use_effect_with_deps(
        {
            let api_dispatch = api_dispatch.clone();
            move |report_id: &Option<String>| {
                if let Some(id) = report_id {
                    api_dispatch.apply(ApiRequestAction::Get(
                        GetParameters {
                            ignore_cache: false,
                        },
                        Gettable::A1Report { id: id.clone() },
                    ));
                }
                || {}
            }
        },
        report_id,
    );
    // Reset fields for new report
    use_effect_with_deps(
        {
            let rpm_string_setter = entered_rpm_state.setter();
            let picked_fan_series_setter = picked_fan_series_state.setter();
            let picked_fan_size_setter = picked_fan_size_state.setter();
            let determinations_setter = determinations_state.setter();
            move |report_option: &Option<A1Standard2010Report<FanSize<FanSeries<()>>>>| {
                if let Some(report) = report_option {
                    let (new_fan_size, new_fan_series): (FanSize<()>, FanSeries<()>) =
                        report.fan_size.clone().into();
                    picked_fan_series_setter.set(Some(new_fan_series));
                    picked_fan_size_setter.set(Some(new_fan_size));
                    rpm_string_setter.set(report.parameters.rpm.to_string());
                    determinations_setter.set(
                        report
                            .determinations
                            .clone()
                            .into_iter()
                            .map(|det| {
                                [
                                    det.static_pressure.to_string(),
                                    det.cfm.to_string(),
                                    det.brake_horsepower.to_string(),
                                ]
                            })
                            .collect(),
                    );
                } else {
                    picked_fan_series_setter.set(None);
                    picked_fan_size_setter.set(None);
                    rpm_string_setter.set("".to_string());
                    determinations_setter.set(vec![])
                }
            }
        },
        maybe_report.clone(),
    );

    let handle_save_callback = {
        use_callback(
            |_evt: MouseEvent, (dispatch, parsed_update_body_ref)| {
                if let Ok(update_body) = parsed_update_body_ref {
                    dispatch.apply(ApiRequestAction::Get(
                        GetParameters { ignore_cache: true },
                        Gettable::PutA12010Report {
                            body: update_body.clone(),
                        },
                    ))
                }
            },
            (api_dispatch.clone(), parsed_update_body.clone()),
        )
    };

    let on_report_id_change = {
        let report_id_setter = report_id_state.setter();
        use_callback(
            move |(_index, report_id), _deps| report_id_setter.set(report_id),
            (),
        )
    };

    let on_rpm_input_change = {
        let entered_rpm_setter = entered_rpm_state.setter();
        use_callback(
            move |(_index, rpm_option), _deps| entered_rpm_setter.set(rpm_option),
            (),
        )
    };

    let on_dets_input_change = {
        let determinations_table_setter = determinations_state.setter();
        use_callback(
            move |data: [[String; 3]; 10], _dets| determinations_table_setter.set(data.to_vec()),
            (),
        )
    };

    let on_determinations_extracted = {
        let determinations_setter = determinations_state.setter();
        use_callback(
            move |dets: Vec<[String; 3]>, _deps| determinations_setter.set(dets),
            (),
        )
    };

    let saved_size = maybe_report.as_ref().map(|report| {
        let (fan_size, _fan_series): (FanSize<()>, FanSeries<()>) = report.fan_size.clone().into();
        fan_size
    });

    let (header, maybe_test_id_input) = match &maybe_report {
        None => {
            let test_id_input = html! {
                <>
                    <label>{"Report ID"}</label>
                    <TaggedInput<()>
                        value={(*report_id_state).clone()}
                        tag={()}
                        onchange={on_report_id_change}
                    />
                </>
            };
            (html! {<h1>{"New A1 Report"}</h1>}, Some(test_id_input))
        }
        Some(report) => (html! {<h1>{"Test No. "}{ report.id.to_owned() }</h1>}, None),
    };

    let det_errs = parsed_update_body
        .clone()
        .err()
        .map(|e| e.determination_errs)
        .unwrap_or_default();

    let maybe_points_to_render = parsed_update_body
        .clone()
        .map(|u| Some(u.determinations))
        .unwrap_or(maybe_report.clone().map(|r| r.determinations));

    html! {
        <>
            {header}
            <div style="display: flex; flex-direction: row;">
                <form>
                    <h2>{"Test Details"}</h2>
                    <div style="display: grid; grid-template-columns: auto auto; width: fit-content; column-gap: 8px; row-gap: 4px;">
                        {maybe_test_id_input}
                        <FanSeriesAndSizePicker
                            size_errs={parsed_update_body.clone().err().map(|e| e.size_errs).unwrap_or_default()}
                            {saved_size}
                            {picked_fan_series_state}
                            {picked_fan_size_state}
                        />
                        <label>{"Test RPM"}</label>
                        <TaggedInput<()>
                            errs={parsed_update_body.clone().err().map(|e| e.rpm_errs).unwrap_or_default()}
                            value={(*entered_rpm_state).clone()}
                            tag={()}
                            onchange={on_rpm_input_change}
                        />
                    </div>
                    <label><h2>{"Determination Points"}</h2></label>
                    <A12010DeterminationTable
                        fields={(*determinations_state).clone()}
                        child_errs={det_errs}
                        onchange={on_dets_input_change}
                    />
                    <label><h3>{"Quick Paste Determination Points"}</h3></label>
                    <DeterminationsPasteTextArea<3,10>
                        on_extracted={on_determinations_extracted}
                        cols_to_extract={[3,4,5]}
                        expected_row_length={9}
                        expected_headers={vec![
                            "Det. No. P t P v P s Q H K p η t η s",
                            "(in. wg) (in. wg) (in. wg) (cfm) (hp) - (%) (%)"
                        ]}
                    />
                    <button
                    //disabled={parsed_update_body.clone().is_err()}
                    onclick={handle_save_callback}>{"Save"}</button>
                </form>
                <div style="flex-grow: 1">
                    { maybe_points_to_render
                        .map(|fc| html!{ <A1FanPlot points={fc} /> })
                        .unwrap_or(html! { <p>{"Once you enter a complete fan curve, you'll see it here"}</p> }) 
                    }
                </div>
            </div>
        </>
    }
}
